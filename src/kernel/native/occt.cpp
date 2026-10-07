// Exact feature evaluation and cap provenance stay inside worker-owned native
// caches; Rust receives only owned derived results.
#include "confusion/src/kernel/bridge.rs.h"
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepOffsetAPI_DraftAngle.hxx>
#include <BRepAlgoAPI_Defeaturing.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepAlgoAPI_Section.hxx>
#include <BRepFeat_SplitShape.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <HLRBRep_Algo.hxx>
#include <HLRBRep_HLRToShape.hxx>
#include <HLRAlgo_Projector.hxx>
#include <BRepLib.hxx>
#include <ShapeAnalysis_FreeBounds.hxx>
#include <TopTools_HSequenceOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>

#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepOffsetAPI_MakePipe.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <BRepLib.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Poly_Triangulation.hxx>
#include <Standard_Failure.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ListIteratorOfListOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <algorithm>
#include <cmath>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <stdexcept>
#include <vector>
#include "inspect.hpp"
namespace confusion {
constexpr double tau = 6.2831853071795864769;
static void check_depth(double depth) {
  if (!std::isfinite(depth) || depth <= 1e-7 || depth > 1000)
    throw std::runtime_error(
        "Extrusion depth must be positive and within modeling limits");
}
static void check_solid(const TopoDS_Shape &shape) {
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
    throw std::runtime_error("Feature produced an invalid solid");
  unsigned solids = 0;
  for (TopExp_Explorer i(shape, TopAbs_SOLID); i.More(); i.Next())
    ++solids;
  if (solids != 1)
    throw std::runtime_error("Feature must produce one connected solid; repair "
                             "the region or target");
}
static bool has_face(const TopoDS_Shape &shape, const TopoDS_Shape &face) {
  for (TopExp_Explorer i(shape, TopAbs_FACE); i.More(); i.Next())
    if (i.Current().IsSame(face))
      return true;
  return false;
}
struct Cap {
  uint32_t producer;
  uint32_t role;
  std::vector<TopoDS_Shape> faces;
};
struct Output {
  TopoDS_Shape shape;
  std::vector<Cap> caps;
};
// OCCT algorithms can update shared topology. Store detached shapes so later
// operations cannot mutate previously cached results (including cap identities).
static Output detached(const Output &source) {
  BRepBuilderAPI_Copy copy(source.shape, true, false);
  Output output{copy.Shape(), source.caps};
  for (auto &cap : output.caps) for (auto &face : cap.faces)
    face = copy.ModifiedShape(face);
  return output;
}
static std::vector<Output> detached(const std::vector<Output> &sources) {
  std::vector<Output> result;
  for (const auto &source : sources) result.push_back(detached(source));
  return result;
}
// Cache entries own native shapes on the evaluation worker. Keys describe inputs,
// never edit history. One entry per feature bounds retention across edits.
struct FeatureEntry {
  std::string key;
  Output output;
  std::vector<std::pair<size_t, TopoDS_Shape>> pieces;
  std::vector<size_t> consumed_targets;
};
struct EditEntry {
  std::string key;
  std::vector<Output> outputs;
  std::vector<bool> consumed;
};
struct ModelCacheImpl {
  std::vector<FeatureEntry> features;
  std::vector<EditEntry> edits;
  size_t reused = 0;
};
ModelCache::ModelCache() : impl(std::make_unique<ModelCacheImpl>()) {}
ModelCache::~ModelCache() = default;
size_t ModelCache::reused_features() const { return impl->reused; }
std::unique_ptr<ModelCache> new_model_cache() { return std::make_unique<ModelCache>(); }
#include "modify.inc"
static Mesh mesh_shape(const TopoDS_Shape &shape,
                       const std::vector<Output> &outputs = {},
                       const std::vector<bool> &consumed = {},
                       const std::vector<std::pair<size_t, TopoDS_Shape>> &pieces = {}) {
  if (!BRepCheck_Analyzer(shape).IsValid())
    throw std::runtime_error("Invalid final model");
  GProp_GProps properties;
  BRepGProp::VolumeProperties(shape, properties);
  Mesh result;
  result.volume = properties.Mass() * 1e-9;
  result.faces = 0;
  result.inspection = inspect_shape(shape);
  if (!std::isfinite(result.volume) || result.volume < 0)
    throw std::runtime_error("Model has no solid volume");
  BRepMesh_IncrementalMesh mesher(shape, 0.1, false, 0.3, true);
  if (!mesher.IsDone())
    throw std::runtime_error("Could not triangulate solid");
  for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next()) {
    const auto face = TopoDS::Face(it.Current());
    TopLoc_Location location;
    auto triangles = BRep_Tool::Triangulation(face, location);
    if (triangles.IsNull())
      throw std::runtime_error("Missing face triangulation");
    ++result.faces;
    for (size_t body = 0; body < outputs.size(); ++body)
      if (!consumed[body] && has_face(outputs[body].shape, face))
        result.bodies.push_back(BodyFace{result.faces, static_cast<uint32_t>(body), 0});
    for (const auto &piece : pieces)
      if (has_face(piece.second, face))
        result.bodies.push_back(BodyFace{result.faces, static_cast<uint32_t>(piece.first), 1});
    for (size_t support = 0; support < outputs.size(); ++support) {
      if (consumed[support])
        continue;
      for (const auto &cap : outputs[support].caps)
        for (const auto &f : cap.faces)
          if (f.IsSame(face))
            result.anchors.push_back(
                FaceAnchor{result.faces, static_cast<uint32_t>(support),
                           cap.producer, cap.role, cap.faces.size() != 1});
    }
    for (int t = 1; t <= triangles->NbTriangles(); ++t) {
      int a, b, c;
      triangles->Triangle(t).Get(a, b, c);
      if (face.Orientation() == TopAbs_REVERSED)
        std::swap(b, c);
      auto pa = triangles->Node(a).Transformed(location.Transformation());
      auto pb = triangles->Node(b).Transformed(location.Transformation());
      auto pc = triangles->Node(c).Transformed(location.Transformation());
      auto normal = gp_Vec(pa, pb).Crossed(gp_Vec(pa, pc));
      if (normal.SquareMagnitude() < 1e-20)
        continue;
      normal.Normalize();
      for (auto p : {pa, pb, pc}) {
        result.indices.push_back(static_cast<uint32_t>(result.vertices.size()));
        result.vertices.push_back(Vertex{p.X() * 0.001, p.Y() * 0.001,
                                         p.Z() * 0.001, normal.X(), normal.Y(),
                                         normal.Z(), result.faces});
      }
    }
  }
  return result;
}
static TopoDS_Face profile_face(rust::Slice<const ProfileEdge> edges) {
  if (edges.size() == 0)
    throw std::runtime_error("Empty profile");
  BRepBuilderAPI_MakeFace face;
  size_t index = 0;
  uint32_t wireIndex = 0;
  while (index < edges.size()) {
    BRepBuilderAPI_MakeWire wire;
    while (index < edges.size() && edges[index].wire == wireIndex) {
      const auto &e = edges[index++];
      for (double v : {e.sx, e.sy, e.ex, e.ey, e.cx, e.cy, e.sweep})
        if (!std::isfinite(v))
          throw std::runtime_error("Invalid profile coordinate");
      gp_Pnt a(e.sx * 1000, e.sy * 1000, 0), b(e.ex * 1000, e.ey * 1000, 0);
      if (e.sweep == 0) {
        BRepBuilderAPI_MakeEdge edge(a, b);
        if (!edge.IsDone())
          throw std::runtime_error("Invalid line edge");
        wire.Add(edge.Edge());
      } else {
        double radius = std::hypot(e.sx - e.cx, e.sy - e.cy) * 1000;
        if (radius <= 1e-4 || std::abs(e.sweep) > tau + 1e-8)
          throw std::runtime_error("Invalid circular edge");
        gp_Circ circle(
            gp_Ax2(gp_Pnt(e.cx * 1000, e.cy * 1000, 0), gp_Dir(0, 0, 1)),
            radius);
        double start = std::atan2(e.sy - e.cy, e.sx - e.cx),
               end = start + e.sweep;
        BRepBuilderAPI_MakeEdge maker(circle, std::min(start, end),
                                      std::max(start, end));
        if (!maker.IsDone())
          throw std::runtime_error("Invalid arc edge");
        auto edge = maker.Edge();
        if (e.sweep < 0)
          edge.Reverse();
        wire.Add(edge);
      }
      if (!wire.IsDone())
        throw std::runtime_error("Could not connect profile edges");
    }
    if (!wire.IsDone() || !wire.Wire().Closed())
      throw std::runtime_error("Profile wire is open");
    if (wireIndex == 0)
      face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)),
                                     wire.Wire(), true);
    else
      face.Add(wire.Wire());
    ++wireIndex;
    if (index < edges.size() && edges[index].wire != wireIndex)
      throw std::runtime_error("Invalid profile wire order");
  }
  if (!face.IsDone() || !BRepCheck_Analyzer(face.Face()).IsValid())
    throw std::runtime_error("Invalid planar profile");
  return face.Face();
}
static gp_Trsf attachment(const ModelStep &step,
                          const std::vector<Output> &outputs) {
  gp_Trsf transform;
  if (step.support < 0) {
    if (step.producer != -1 || step.role != 0)
      throw std::runtime_error("Invalid origin plane");
    return transform;
  }
  if (static_cast<size_t>(step.support) >= outputs.size() || step.producer < 0)
    throw std::runtime_error("Missing sketch support");
  const Cap *selected = nullptr;
  for (const auto &cap : outputs[step.support].caps)
    if (cap.producer == static_cast<uint32_t>(step.producer) &&
        cap.role == step.role) {
      if (selected)
        throw std::runtime_error("Ambiguous face attachment; reattach sketch");
      selected = &cap;
    }
  if (!selected || selected->faces.empty())
    throw std::runtime_error(
        "Sketch support face was deleted; reattach sketch");
  if (selected->faces.size() != 1)
    throw std::runtime_error("Sketch support face split; reattach sketch");
  const auto face = TopoDS::Face(selected->faces[0]);
  BRepAdaptor_Surface surface(face);
  if (surface.GetType() != GeomAbs_Plane)
    throw std::runtime_error("Sketch support is not planar");
  auto plane = surface.Plane();
  auto normal = plane.Axis().Direction();
  if (face.Orientation() == TopAbs_REVERSED)
    normal.Reverse();
  // Preserve the producing plane's local origin and X axis as dimensions
  // change.
  auto x = plane.Position().XDirection();
  auto y = normal.Crossed(x);
  auto o = plane.Location();
  transform.SetValues(x.X(), y.X(), normal.X(), o.X(), x.Y(), y.Y(), normal.Y(),
                      o.Y(), x.Z(), y.Z(), normal.Z(), o.Z());
  return transform;
}
template <class Boolean>
static std::vector<Cap> follow_caps(Boolean &operation,
                                    const std::vector<Cap> &input,
                                    const TopoDS_Shape &result) {
  std::vector<Cap> caps;
  for (const auto &source : input) {
    Cap cap{source.producer, source.role, {}};
    auto add = [&](const TopoDS_Shape &shape) {
      if (shape.ShapeType() != TopAbs_FACE || !has_face(result, shape))
        return;
      for (const auto &old : cap.faces)
        if (old.IsSame(shape))
          return;
      // Obtain the orientation from the result, particularly for cut tool
      // faces.
      for (TopExp_Explorer i(result, TopAbs_FACE); i.More(); i.Next())
        if (i.Current().IsSame(shape)) {
          cap.faces.push_back(i.Current());
          break;
        }
    };
    for (const auto &face : source.faces) {
      if (!operation.IsDeleted(face))
        add(face);
      for (TopTools_ListIteratorOfListOfShape i(operation.Modified(face));
           i.More(); i.Next())
        add(i.Value());
      for (TopTools_ListIteratorOfListOfShape i(operation.Generated(face));
           i.More(); i.Next())
        add(i.Value());
    }
    caps.push_back(std::move(cap));
  }
  return caps;
}
#include "src/kernel/native/solid_create.inc"
static Mesh evaluate_impl(ModelCacheImpl *cache, rust::Slice<const rust::String> keys, rust::Slice<const ProfileEdge> edges,
                    rust::Slice<const ModelStep> steps,
                    rust::Slice<const FaceRequest> planes, rust::Slice<const ModifyStep> edits, rust::Slice<const CreateStep> creates) {
  try {
    if (steps.size() + creates.size() == 0 || steps.size() > 64 || creates.size() > 64)
      throw std::runtime_error("Invalid model feature count");
    if (cache) {
      if (keys.size() != steps.size() + creates.size() + edits.size()) throw std::runtime_error("Invalid cache keys");
      cache->reused = 0;
      cache->features.resize(steps.size() + creates.size());
      cache->edits.resize(edits.size());
    }
    std::vector<Output> outputs;
    std::vector<bool> consumed;
    std::vector<std::pair<size_t, TopoDS_Shape>> pieces;
    std::vector<gp_Trsf> frames;
    for (size_t index = 0; index < steps.size(); ++index) {
      const auto &step = steps[index];
      FeatureEntry *entry = cache ? &cache->features[index] : nullptr;
      if (entry && entry->key == std::string(keys[index])) {
        if (step.operation != 0 && (step.target < 0 ||
            static_cast<size_t>(step.target) >= consumed.size() || consumed[step.target]))
          throw std::runtime_error("Missing or consumed target body");
        outputs.push_back(detached(entry->output));
        consumed.push_back(false);
        if (step.operation != 0) consumed.at(step.target) = true;
        for (const auto &piece : entry->pieces) {
          BRepBuilderAPI_Copy copy(piece.second, true, false);
          pieces.emplace_back(piece.first, copy.Shape());
        }
        ++cache->reused;
        continue;
      }
      const auto piece_start = pieces.size();
      try {
        check_depth(step.depth);
        if (step.edge_count == 0 || step.edge_start > edges.size() ||
            step.edge_count > edges.size() - step.edge_start)
          throw std::runtime_error("Invalid profile range");
        if (step.operation > 3)
          throw std::runtime_error("Unknown extrusion operation");
        auto transform = attachment(step, outputs);
        auto local = profile_face(rust::Slice<const ProfileEdge>(
            edges.data() + step.edge_start, step.edge_count));
        BRepBuilderAPI_Transform place(local, transform, true);
        if (!place.IsDone())
          throw std::runtime_error("Could not place sketch plane");
        auto face = TopoDS::Face(place.Shape());
        gp_Vec direction(0, 0,
                         (step.operation >= 2 ? -1 : 1) * step.depth * 1000);
        direction.Transform(transform);
        BRepPrimAPI_MakePrism prism(face, direction);
        if (!prism.IsDone())
          throw std::runtime_error("Extrusion failed");
        Output output{prism.Shape(),
                      {{static_cast<uint32_t>(index), 1, {prism.FirstShape()}},
                       {static_cast<uint32_t>(index), 2, {prism.LastShape()}}}};
        check_solid(output.shape);
        // Prism history can return the source orientation. Use the outward
        // orientation of the corresponding face in the evaluated solid for
        // sketch frames.
        for (auto &cap : output.caps)
          for (auto &face : cap.faces) {
            for (TopExp_Explorer it(output.shape, TopAbs_FACE); it.More();
                 it.Next())
              if (it.Current().IsSame(face)) {
                face = it.Current();
                break;
              }
          }
        if (step.operation == 0) {
          if (step.target != -1)
            throw std::runtime_error("New body cannot have a target");
        } else {
          if (step.target < 0 ||
              static_cast<size_t>(step.target) >= outputs.size() ||
              consumed[step.target])
            throw std::runtime_error("Missing or consumed target body");
          auto caps = outputs[step.target].caps;
          caps.insert(caps.end(), output.caps.begin(), output.caps.end());
          GProp_GProps before, after;
          BRepGProp::VolumeProperties(outputs[step.target].shape, before);
          if (step.operation == 1) {
            BRepAlgoAPI_Fuse op(outputs[step.target].shape, output.shape);
            op.Build();
            if (!op.IsDone() || op.HasErrors())
              throw std::runtime_error("Join failed");
            output.shape = op.Shape();
            output.caps = follow_caps(op, caps, output.shape);
          } else {
            if (step.operation == 3) {
              BRepAlgoAPI_Common common(outputs[step.target].shape, output.shape);
              common.Build();
              if (!common.IsDone() || common.HasErrors()) throw std::runtime_error("Could not create the cut body");
              check_solid(common.Shape());
              pieces.emplace_back(index, common.Shape());
            }
            BRepAlgoAPI_Cut op(outputs[step.target].shape, output.shape);
            op.Build();
            if (!op.IsDone() || op.HasErrors())
              throw std::runtime_error("Cut failed");
            output.shape = op.Shape();
            output.caps = follow_caps(op, caps, output.shape);
          }
          check_solid(output.shape);
          BRepGProp::VolumeProperties(output.shape, after);
          if (std::abs(after.Mass() - before.Mass()) < 1e-6)
            throw std::runtime_error("Feature does not change the target body");
          consumed[step.target] = true;
        }
        if (entry) {
          entry->key.clear();
          entry->output = detached(output);
          entry->pieces.clear();
          for (size_t p = piece_start; p < pieces.size(); ++p) {
            BRepBuilderAPI_Copy copy(pieces[p].second, true, false);
            entry->pieces.emplace_back(pieces[p].first, copy.Shape());
          }
          entry->key = std::string(keys[index]);
        }
        outputs.push_back(std::move(output));
        consumed.push_back(false);
      } catch (const Standard_Failure &e) {
        throw std::runtime_error("Feature " + std::to_string(index + 1) + ": " +
                                 e.GetMessageString());
      } catch (const std::exception &e) {
        throw std::runtime_error("Feature " + std::to_string(index + 1) + ": " +
                                 e.what());
      }
    }
    append_creates(edges, creates, outputs, consumed, cache, keys, steps.size());
    for (size_t i = 0; i < planes.size(); ++i) {
      ModelStep plane{};
      plane.support = planes[i].support;
      plane.producer = planes[i].producer;
      plane.role = planes[i].role;
      try {
        frames.push_back(attachment(plane, outputs));
      } catch (const std::exception &e) {
        throw std::runtime_error("Sketch support " + std::to_string(i + 1) +
                                 ": " + e.what());
      }
    }
    apply_edits(outputs, consumed, edits, cache, keys, steps.size() + creates.size());
    TopoDS_Compound combined;
    BRep_Builder builder;
    builder.MakeCompound(combined);
    for (size_t i = 0; i < outputs.size(); ++i)
      if (!consumed[i])
        builder.Add(combined, outputs[i].shape);
    for (const auto &piece : pieces) builder.Add(combined, piece.second);
    auto result = mesh_shape(combined, outputs, consumed, pieces);
    std::vector<TopoDS_Shape> inspection_bodies;
    for (size_t i = 0; i < outputs.size(); ++i)
      if (!consumed[i]) inspection_bodies.push_back(outputs[i].shape);
    for (const auto &piece : pieces) inspection_bodies.push_back(piece.second);
    inspect_interference(inspection_bodies, result.inspection);

    for (const auto &frame : frames) {
      auto o = frame.TranslationPart();
      auto x = gp_Vec(1,0,0).Transformed(frame);
      auto y = gp_Vec(0,1,0).Transformed(frame);
      auto n = gp_Vec(0,0,1).Transformed(frame);
      result.planes.push_back(PlaneFrame{o.X()*0.001,o.Y()*0.001,o.Z()*0.001,
        x.X(),x.Y(),x.Z(),y.X(),y.Y(),y.Z(),n.X(),n.Y(),n.Z()});
    }
    return result;
  } catch (const Standard_Failure &e) {
    throw std::runtime_error(e.GetMessageString());
  }
}
Mesh evaluate_complete_model(rust::Slice<const ProfileEdge> edges, rust::Slice<const ModelStep> steps,
    rust::Slice<const FaceRequest> planes, rust::Slice<const ModifyStep> edits, rust::Slice<const CreateStep> creates) {
  return evaluate_impl(nullptr, {}, edges, steps, planes, edits, creates);
}
Mesh evaluate_cached_model(ModelCache &cache, rust::Slice<const ProfileEdge> edges, rust::Slice<const ModelStep> steps,
    rust::Slice<const FaceRequest> planes, rust::Slice<const ModifyStep> edits, rust::Slice<const CreateStep> creates,
    rust::Slice<const rust::String> keys) {
  return evaluate_impl(cache.impl.get(), keys, edges, steps, planes, edits, creates);
}
Mesh evaluate_create_model(rust::Slice<const ProfileEdge> edges, rust::Slice<const ModelStep> steps,
                    rust::Slice<const FaceRequest> planes,
                    rust::Slice<const CreateStep> creates) {
  return evaluate_complete_model(edges,steps,planes,rust::Slice<const ModifyStep>(),creates);
}
Mesh evaluate_modified_model(rust::Slice<const ProfileEdge> edges, rust::Slice<const ModelStep> steps,
                    rust::Slice<const FaceRequest> planes, rust::Slice<const ModifyStep> edits) {
  return evaluate_complete_model(edges,steps,planes,edits,rust::Slice<const CreateStep>());
}
Mesh evaluate_model(rust::Slice<const ProfileEdge> edges,
                    rust::Slice<const ModelStep> steps,
                    rust::Slice<const FaceRequest> planes) {
  return evaluate_create_model(edges, steps, planes, rust::Slice<const CreateStep>());
}
Mesh extrude_region(rust::Slice<const ProfileEdge> edges, double depth) {
  try {
    check_depth(depth);
    BRepPrimAPI_MakePrism prism(profile_face(edges),
                                gp_Vec(0, 0, depth * 1000));
    if (!prism.IsDone())
      throw std::runtime_error("Extrusion failed");
    check_solid(prism.Shape());
    return mesh_shape(prism.Shape());
  } catch (const Standard_Failure &e) {
    throw std::runtime_error(e.GetMessageString());
  }
}
Mesh extrude(rust::Slice<const Point2> points, double depth) {
  try {
    check_depth(depth);
    if (points.size() < 3)
      throw std::runtime_error("Invalid polygon");
    BRepBuilderAPI_MakePolygon polygon;
    for (const auto &p : points) {
      if (!std::isfinite(p.x) || !std::isfinite(p.y))
        throw std::runtime_error("Invalid profile coordinate");
      polygon.Add(gp_Pnt(p.x * 1000, p.y * 1000, 0));
    }
    polygon.Close();
    if (!polygon.IsDone())
      throw std::runtime_error("Could not build profile wire");
    BRepBuilderAPI_MakeFace face(polygon.Wire());
    if (!face.IsDone())
      throw std::runtime_error("Could not build planar face");
    BRepPrimAPI_MakePrism prism(face.Face(), gp_Vec(0, 0, depth * 1000));
    if (!prism.IsDone())
      throw std::runtime_error("Extrusion failed");
    check_solid(prism.Shape());
    return mesh_shape(prism.Shape());
  } catch (const Standard_Failure &e) {
    throw std::runtime_error(e.GetMessageString());
  }
}
} // namespace confusion
