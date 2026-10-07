// Inspection runs on the evaluation worker; native shapes remain local.
#pragma once
#include <BRepAlgoAPI_Common.hxx>
#include <BRepBndLib.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepTools.hxx>
#include <Bnd_Box.hxx>
#include <GeomLProp_SLProps.hxx>
#include <gp_Pnt2d.hxx>
#include <limits>
namespace confusion {
static Inspection inspect_shape(const TopoDS_Shape &shape) {
  Inspection result{};
  result.valid = BRepCheck_Analyzer(shape).IsValid();
  for (TopExp_Explorer it(shape, TopAbs_SOLID); it.More(); it.Next()) ++result.solids;
  GProp_GProps volume, surface;
  BRepGProp::VolumeProperties(shape, volume);
  BRepGProp::SurfaceProperties(shape, surface);
  result.area = surface.Mass() * 1e-6;
  auto center = volume.CentreOfMass();
  result.cx = center.X() * 0.001;
  result.cy = center.Y() * 0.001;
  result.cz = center.Z() * 0.001;
  uint32_t id = 0;
  for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next()) {
    const auto face = TopoDS::Face(it.Current());
    FaceInspection entry{};
    entry.face = ++id;
    GProp_GProps area;
    BRepGProp::SurfaceProperties(face, area);
    entry.area = area.Mass() * 1e-6;
    entry.min_curvature = entry.min_draft = std::numeric_limits<double>::infinity();
    entry.max_curvature = entry.max_draft = -std::numeric_limits<double>::infinity();
    double u0, u1, v0, v1;
    BRepTools::UVBounds(face, u0, u1, v0, v1);
    TopLoc_Location location;
    auto geometry = BRep_Tool::Surface(face, location);
    // Interior grid samples, classified against trimming wires (including holes).
    for (int u = 0; u < 5; ++u) for (int v = 0; v < 5; ++v) {
      double x = u0 + (u + 0.5) * (u1 - u0) / 5;
      double y = v0 + (v + 0.5) * (v1 - v0) / 5;
      if (!std::isfinite(x) || !std::isfinite(y)) continue;
      BRepClass_FaceClassifier classifier(face, gp_Pnt2d(x, y), 1e-7);
      if (classifier.State() != TopAbs_IN) continue;
      try {
        GeomLProp_SLProps properties(geometry, x, y, 2, 1e-7);
        if (!properties.IsNormalDefined() || !properties.IsCurvatureDefined()) continue;
        auto normal = properties.Normal();
        normal.Transform(location.Transformation());
        double sign = face.Orientation() == TopAbs_REVERSED ? -1 : 1;
        double draft = std::asin(std::clamp(sign * normal.Z(), -1.0, 1.0)) * 180 / 3.141592653589793;
        // Curvature magnitude is independent of face orientation; SI inverse metres.
        double k0 = std::abs(properties.MinCurvature()) * 1000;
        double k1 = std::abs(properties.MaxCurvature()) * 1000;
        entry.min_curvature = std::min(entry.min_curvature, std::min(k0, k1));
        entry.max_curvature = std::max(entry.max_curvature, std::max(k0, k1));
        entry.min_draft = std::min(entry.min_draft, draft);
        entry.max_draft = std::max(entry.max_draft, draft);
        ++entry.samples;
      } catch (const Standard_Failure &) { /* Singular samples are reported as unavailable. */ }
    }
    result.faces.push_back(entry);
  }
  return result;
}
static void inspect_interference(const std::vector<TopoDS_Shape> &bodies, Inspection &result) {
  std::vector<Bnd_Box> bounds(bodies.size());
  for (size_t i = 0; i < bodies.size(); ++i) BRepBndLib::Add(bodies[i], bounds[i]);
  for (size_t i = 0; i < bodies.size(); ++i) for (size_t j = i + 1; j < bodies.size(); ++j) {
    if (bounds[i].IsOut(bounds[j])) continue;
    try {
      BRepAlgoAPI_Common common(bodies[i], bodies[j]);
      common.Build();
      if (!common.IsDone() || common.HasErrors()) {
        result.error = "Some body intersections could not be evaluated";
        continue;
      }
      GProp_GProps volume;
      BRepGProp::VolumeProperties(common.Shape(), volume);
      if (volume.Mass() > 1e-6)
        result.interference.push_back(Interference{static_cast<uint32_t>(i + 1),
            static_cast<uint32_t>(j + 1), volume.Mass() * 1e-9});
    } catch (const Standard_Failure &) {
      result.error = "Some body intersections could not be evaluated";
    }
  }
}
} // namespace confusion
