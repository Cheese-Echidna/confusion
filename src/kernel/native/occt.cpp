// Build a checked exact B-rep prism, then derive an indexed face-ID mesh for the GPU.
// Connects only to kernel/bridge.rs. No shape pointer or triangulation is serialized.
#include "confusion/src/kernel/bridge.rs.h"
#include <BRepBuilderAPI_MakePolygon.hxx>
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
#include <cmath>
#include <stdexcept>
namespace confusion {
Mesh extrude(rust::Slice<const Point2> points,double depth) {
  try {
    if(points.size()<3 || !std::isfinite(depth) || depth<=1e-7 || depth>1000) throw std::runtime_error("Extrusion depth must be positive");
    // OCCT operates in millimetres here for robust default tolerances; the API is metres.
    BRepBuilderAPI_MakePolygon polygon;
    for(const auto &p:points) {if(!std::isfinite(p.x)||!std::isfinite(p.y))throw std::runtime_error("Invalid profile coordinate");polygon.Add(gp_Pnt(p.x*1000,p.y*1000,0));}
    polygon.Close(); if(!polygon.IsDone())throw std::runtime_error("Could not build profile wire");
    BRepBuilderAPI_MakeFace face(polygon.Wire()); if(!face.IsDone())throw std::runtime_error("Could not build planar face");
    BRepPrimAPI_MakePrism prism(face.Face(),gp_Vec(0,0,depth*1000));
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
  } catch(const Standard_Failure &e){throw std::runtime_error(e.GetMessageString());}
}
}
