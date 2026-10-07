// Build a checked exact B-rep prism, then derive an indexed face-ID mesh for the GPU.
// Connects only to kernel/bridge.rs. No shape pointer or triangulation is serialized.
#include "confusion/src/kernel/bridge.rs.h"
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <gp_Circ.hxx>
#include <gp_Ax2.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <Poly_Triangulation.hxx>
#include <Standard_Failure.hxx>
#include <gp_Vec.hxx>
#include <gp_Pln.hxx>
#include <cmath>
#include <stdexcept>
namespace confusion {
static Mesh extrude_face(const TopoDS_Face &face, double depth) {
    BRepPrimAPI_MakePrism prism(face,gp_Vec(0,0,depth*1000));
    if(!prism.IsDone())throw std::runtime_error("Extrusion failed");
    auto shape=prism.Shape(); if(!BRepCheck_Analyzer(shape).IsValid())throw std::runtime_error("Extrusion produced an invalid solid");
    GProp_GProps properties; BRepGProp::VolumeProperties(shape,properties);
    Mesh result; result.volume=properties.Mass()*1e-9;result.faces=0;
    if(result.volume<=0)throw std::runtime_error("Extrusion has no solid volume");
    BRepMesh_IncrementalMesh mesher(shape,0.1,false,0.3,true);
    if(!mesher.IsDone())throw std::runtime_error("Could not triangulate solid");
    for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
      const auto face=TopoDS::Face(it.Current());TopLoc_Location location;
      auto triangles=BRep_Tool::Triangulation(face,location);if(triangles.IsNull())throw std::runtime_error("Missing face triangulation");
      ++result.faces;
      for(int t=1;t<=triangles->NbTriangles();++t){int a,b,c;triangles->Triangle(t).Get(a,b,c);if(face.Orientation()==TopAbs_REVERSED)std::swap(b,c);
        auto pa=triangles->Node(a).Transformed(location.Transformation());auto pb=triangles->Node(b).Transformed(location.Transformation());auto pc=triangles->Node(c).Transformed(location.Transformation());
        auto normal=gp_Vec(pa,pb).Crossed(gp_Vec(pa,pc));if(normal.SquareMagnitude()<1e-20)continue;normal.Normalize();
        for(auto p:{pa,pb,pc}){result.indices.push_back(static_cast<uint32_t>(result.vertices.size()));result.vertices.push_back(Vertex{p.X()*0.001,p.Y()*0.001,p.Z()*0.001,normal.X(),normal.Y(),normal.Z(),result.faces});}
      }
    }
    return result;
}
Mesh extrude(rust::Slice<const Point2> points,double depth) {
  try {
    if(points.size()<3 || !std::isfinite(depth) || depth<=1e-7 || depth>1000) throw std::runtime_error("Extrusion depth must be positive");
    // OCCT operates in millimetres here for robust default tolerances; the API is metres.
    BRepBuilderAPI_MakePolygon polygon;
    for(const auto &p:points) {if(!std::isfinite(p.x)||!std::isfinite(p.y))throw std::runtime_error("Invalid profile coordinate");polygon.Add(gp_Pnt(p.x*1000,p.y*1000,0));}
    polygon.Close(); if(!polygon.IsDone())throw std::runtime_error("Could not build profile wire");
    BRepBuilderAPI_MakeFace face(polygon.Wire()); if(!face.IsDone())throw std::runtime_error("Could not build planar face");
    return extrude_face(face.Face(),depth);
  } catch(const Standard_Failure &e){throw std::runtime_error(e.GetMessageString());}
}
}

namespace confusion {
Mesh extrude_region(rust::Slice<const ProfileEdge> edges, double depth) {
  try {
    if(edges.size()==0 || !std::isfinite(depth) || depth<=1e-7 || depth>1000) throw std::runtime_error("Extrusion depth must be positive");
    BRepBuilderAPI_MakeFace face;
    size_t index=0; uint32_t wireIndex=0;
    while(index<edges.size()) {
      BRepBuilderAPI_MakeWire wire;
      while(index<edges.size() && edges[index].wire==wireIndex) {
        const auto &e=edges[index++];
        for(double v:{e.sx,e.sy,e.ex,e.ey,e.cx,e.cy,e.sweep}) if(!std::isfinite(v)) throw std::runtime_error("Invalid profile coordinate");
        gp_Pnt a(e.sx*1000,e.sy*1000,0), b(e.ex*1000,e.ey*1000,0);
        if(e.sweep==0) { BRepBuilderAPI_MakeEdge edge(a,b); if(!edge.IsDone()) throw std::runtime_error("Invalid line edge"); wire.Add(edge.Edge()); }
        else {
          double radius=std::hypot(e.sx-e.cx,e.sy-e.cy)*1000;
          if(radius<=1e-4 || std::abs(e.sweep)>2*M_PI+1e-8) throw std::runtime_error("Invalid circular edge");
          gp_Circ circle(gp_Ax2(gp_Pnt(e.cx*1000,e.cy*1000,0),gp_Dir(0,0,1)),radius);
          double start=std::atan2(e.sy-e.cy,e.sx-e.cx), end=start+e.sweep;
          BRepBuilderAPI_MakeEdge maker(circle,std::min(start,end),std::max(start,end));
          if(!maker.IsDone()) throw std::runtime_error("Invalid arc edge");
          auto edge=maker.Edge(); if(e.sweep<0) edge.Reverse(); wire.Add(edge);
        }
        if(!wire.IsDone()) throw std::runtime_error("Could not connect profile edges");
      }
      if(!wire.IsDone() || !wire.Wire().Closed()) throw std::runtime_error("Profile wire is open");
      if(wireIndex==0) face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),wire.Wire(),true);
      else face.Add(wire.Wire());
      ++wireIndex;
      if(index<edges.size() && edges[index].wire!=wireIndex) throw std::runtime_error("Invalid profile wire order");
    }
    if(!face.IsDone() || !BRepCheck_Analyzer(face.Face()).IsValid()) throw std::runtime_error("Invalid planar profile");
    return extrude_face(face.Face(),depth);
  } catch(const Standard_Failure &e) { throw std::runtime_error(e.GetMessageString()); }
}
}
