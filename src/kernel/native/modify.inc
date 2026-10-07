// Included in occt.cpp; cached shapes remain confined to the owning worker.
static TopoDS_Face edit_face(const std::vector<Output>& outputs,
                            const std::vector<bool>& consumed, uint32_t ordinal,
                            uint32_t target) {
  uint32_t count = 0;
  for (size_t i = 0; i < outputs.size(); ++i) {
    if (consumed[i]) continue;
    for (TopExp_Explorer it(outputs[i].shape, TopAbs_FACE); it.More(); it.Next()) {
      if (++count == ordinal) {
        if (i != target) throw std::runtime_error("Selected face belongs to a different body");
        return TopoDS::Face(it.Current());
      }
    }
  }
  throw std::runtime_error("Selected face no longer exists; select it again");
}
static gp_Pln edit_plane(const TopoDS_Face& face) {
  BRepAdaptor_Surface surface(face);
  if (surface.GetType() != GeomAbs_Plane)
    throw std::runtime_error("This operation requires a planar face");
  auto plane = surface.Plane();
  if (face.Orientation() == TopAbs_REVERSED) plane.SetAxis(gp_Ax1(plane.Location(), plane.Axis().Direction().Reversed()));
  return plane;
}
static TopoDS_Shape boolean_edit(const TopoDS_Shape& target, const TopoDS_Shape& tool, uint32_t mode) {
  if (mode == 0) {
    BRepAlgoAPI_Fuse op; boolean_inputs(op,target, tool); build_cancellable(op);
    if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Join failed");
    return op.Shape();
  }
  if (mode == 1) {
    BRepAlgoAPI_Cut op; boolean_inputs(op,target, tool); build_cancellable(op);
    if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Cut failed");
    return op.Shape();
  }
  BRepAlgoAPI_Common op; boolean_inputs(op,target, tool); build_cancellable(op);
  if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Intersection failed");
  return op.Shape();
}
static TopoDS_Shape move_face(const TopoDS_Shape& shape, const TopoDS_Face& face, double distance) {
  if (std::abs(distance) < 1e-7) throw std::runtime_error("Enter a nonzero face displacement");
  auto plane = edit_plane(face);
  BRepPrimAPI_MakePrism prism(face, gp_Vec(plane.Axis().Direction()) * (distance * 1000.));
  if (!prism.IsDone()) throw std::runtime_error("Could not build face displacement");
  return boolean_edit(shape, prism.Shape(), distance > 0 ? 0 : 1);
}
static TopoDS_Shape heal_face(const TopoDS_Shape& shape, const TopoDS_Face& face) {
  // Try the selected cap and adjacent walls as one pocket/boss feature first.
  TopTools_IndexedMapOfShape boundary;TopExp::MapShapes(face,TopAbs_EDGE,boundary);
  BRepAlgoAPI_Defeaturing feature;feature.SetShape(shape);feature.AddFaceToRemove(face);
  for (TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
    if (it.Current().IsSame(face))continue;
    for(TopExp_Explorer edge(it.Current(),TopAbs_EDGE);edge.More();edge.Next()) {
      if(boundary.Contains(edge.Current())) {feature.AddFaceToRemove(it.Current());break;}
    }
  }
  GProp_GProps before, after;BRepGProp::VolumeProperties(shape,before);
  build_cancellable(feature);
  if (feature.IsDone() && !feature.HasErrors() && !has_face(feature.Shape(),face) && BRepCheck_Analyzer(feature.Shape()).IsValid()) {
    BRepGProp::VolumeProperties(feature.Shape(),after);
    if (std::abs(before.Mass()-after.Mass())>1e-6 && after.Mass()>1e-6) return feature.Shape();
  }
  // Blends can be healed by removing just the selected face.
  BRepAlgoAPI_Defeaturing single;single.SetShape(shape);single.AddFaceToRemove(face);build_cancellable(single);
  if(single.IsDone() && !single.HasErrors() && !has_face(single.Shape(),face)) {
    BRepGProp::VolumeProperties(single.Shape(),after);
    if(std::abs(before.Mass()-after.Mass())>1e-6) return single.Shape();
  }
  throw std::runtime_error("Cannot heal this face and its adjacent feature walls");
}
static void apply_edits(std::vector<Output>& outputs, std::vector<bool>& consumed,
                        rust::Slice<const ModifyStep> edits, ModelCacheImpl *cache,
                        rust::Slice<const rust::String> keys, size_t offset) {
  for (size_t n = 0; n < edits.size(); ++n) {
    check_cancelled();
    const auto& e = edits[n];
    auto *entry = cache ? &cache->edits[n] : nullptr;
    if (entry && entry->key == std::string(keys[offset + n])) {
      outputs = detached(entry->outputs);
      consumed = entry->consumed;
      ++cache->reused;
      continue;
    }
    auto remember = [&] {
      if (entry) {
        entry->key.clear();
        entry->outputs = detached(outputs);
        entry->consumed = consumed;
        entry->key = std::string(keys[offset + n]);
      }
    };
    try {
      for(double value : {e.a,e.b,e.c,e.d}) {
        if(!std::isfinite(value) || std::abs(value)>1000.)throw std::runtime_error("Invalid solid edit value");
      }
      if(e.kind>16 || e.mode>2 || e.face>100000)throw std::runtime_error("Invalid solid edit options");
      if((e.kind==1 || e.kind==2 || e.kind==3 || e.kind==5) && e.a<=1e-7)throw std::runtime_error("Radius, thickness or scale factor must be positive");
      if(e.kind==8 && (e.b<1 || std::floor(e.b)!=e.b))throw std::runtime_error("Enter a positive integer replacement face number");
      if (e.target >= outputs.size() || consumed[e.target]) throw std::runtime_error("Missing or removed target body");
      auto shape = outputs[e.target].shape;
      TopoDS_Face face;
      if (e.face) face = edit_face(outputs, consumed, e.face, e.target);
      TopoDS_Shape tool;
      if (e.tool >= 0) {
        if (static_cast<size_t>(e.tool) >= outputs.size() || consumed[e.tool] || static_cast<uint32_t>(e.tool) == e.target)
          throw std::runtime_error("Choose a different current tool body");
        tool = outputs[e.tool].shape;
      }
      TopoDS_Shape result;
      switch (e.kind) {
      case 0: case 7: result = move_face(shape, face, e.a); break;
      case 1: case 2: {
        TopTools_IndexedMapOfShape edges;
        TopExp::MapShapes(face.IsNull() ? shape : face, TopAbs_EDGE, edges);
        if (e.a <= 0) throw std::runtime_error("Radius or distance must be positive");
        if (e.kind == 1) {
          BRepFilletAPI_MakeFillet op(shape);
          for (int i = 1; i <= edges.Extent(); ++i) op.Add(e.a * 1000., TopoDS::Edge(edges(i)));
          build_cancellable(op); if (!op.IsDone()) throw std::runtime_error("Fillet failed; reduce the radius"); result = op.Shape();
        } else {
          BRepFilletAPI_MakeChamfer op(shape);
          for (int i = 1; i <= edges.Extent(); ++i) op.Add(e.a * 1000., TopoDS::Edge(edges(i)));
          build_cancellable(op); if (!op.IsDone()) throw std::runtime_error("Chamfer failed; reduce the distance"); result = op.Shape();
        }
        break;
      }
      case 3: {
        TopTools_ListOfShape openings; openings.Append(face);
        BRepOffsetAPI_MakeThickSolid op;
        run_cancellable([&](const Message_ProgressRange &range) {
          op.MakeThickSolidByJoin(shape, openings, -e.a * 1000., 1e-5, BRepOffset_Skin, false, false, GeomAbs_Intersection, false, range);
        });
        if (!op.IsDone()) throw std::runtime_error("Shell failed; reduce the wall thickness");
        result = op.Shape(); break;
      }
      case 4: {
        BRepOffsetAPI_DraftAngle op(shape);
        op.Add(face, gp_Dir(0,0,1), e.a, gp_Pln(gp_Pnt(0,0,e.b*1000.),gp_Dir(0,0,1)));
        if (!op.AddDone()) throw std::runtime_error("Cannot draft this face along Z");
        build_cancellable(op); if (!op.IsDone()) throw std::runtime_error("Draft failed"); result = op.Shape(); break;
      }
      case 5: case 12: {
        gp_Trsf tr;
        if (e.kind == 5) tr.SetScale(gp_Pnt(e.b*1000.,e.c*1000.,e.d*1000.), e.a);
        else {
          tr.SetRotation(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(0,0,1)), e.d);
          tr.SetTranslationPart(gp_Vec(e.a*1000.,e.b*1000.,e.c*1000.));
        }
        BRepBuilderAPI_Transform op(shape,tr,true);
        if (!op.IsDone()) throw std::runtime_error("Transform failed");
        result = op.Shape(); break;
      }
      case 6: {
        if (tool.IsNull()) throw std::runtime_error("Choose a tool body");
        result = boolean_edit(shape,tool,e.mode);
        if (!e.copy) consumed[e.tool] = true;
        break;
      }
      case 8: {
        if (tool.IsNull()) throw std::runtime_error("Choose a replacement body and face number");
        auto replacement = edit_face(outputs,consumed,static_cast<uint32_t>(e.b),e.tool);
        auto from = edit_plane(face), to = edit_plane(replacement);
        if (std::abs(from.Axis().Direction().Dot(to.Axis().Direction())) < 1.-1e-7)
          throw std::runtime_error("Replace face currently requires parallel planar faces");
        double distance = gp_Vec(from.Location(),to.Location()).Dot(gp_Vec(from.Axis().Direction())) / 1000.;
        result = move_face(shape,face,distance); break;
      }
      case 9: case 10: {
        gp_Vec normal(e.b,e.c,e.d);
        if (normal.Magnitude() < 1e-7) throw std::runtime_error("Enter a nonzero cutting plane normal");
        gp_Pln plane(gp_Pnt(gp_Dir(normal).XYZ()*e.a*1000.),gp_Dir(normal));
        TopoDS_Face cutter = BRepBuilderAPI_MakeFace(plane).Face();
        if (e.kind == 9) {
          TopTools_ListOfShape args, tools; args.Append(shape); tools.Append(cutter);
          BRepAlgoAPI_Splitter op; op.SetArguments(args);op.SetTools(tools);build_cancellable(op);
          if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Body split failed");
          result = op.Shape();
          unsigned solids=0; for(TopExp_Explorer it(result,TopAbs_SOLID);it.More();it.Next()) ++solids;
          if (solids < 2) throw std::runtime_error("Cutting plane does not split the body");
        } else {
          BRepAlgoAPI_Section section(face,cutter,false); section.ComputePCurveOn1(true);build_cancellable(section);
          if (!section.IsDone()) throw std::runtime_error("Face section failed");
          BRepFeat_SplitShape op(shape); unsigned count=0;
          for(TopExp_Explorer it(section.Shape(),TopAbs_EDGE);it.More();it.Next()) { op.Add(TopoDS::Edge(it.Current()),face);++count; }
          if (!count) throw std::runtime_error("Cutting plane does not cross the selected face");
          build_cancellable(op); if (!op.IsDone()) throw std::runtime_error("Face split failed"); result=op.Shape();
        }
        break;
      }
      case 11: {
        if (tool.IsNull()) throw std::runtime_error("Choose a silhouette tool body");
        auto plane=edit_plane(face);
        Handle(HLRBRep_Algo) projection = new HLRBRep_Algo();
        projection->Add(tool);
        projection->Projector(HLRAlgo_Projector(gp_Ax2(plane.Location(),plane.Axis().Direction(),plane.XAxis().Direction())));
        projection->Update();projection->Hide();
        HLRBRep_HLRToShape extract(projection);
        auto outline=extract.VCompound();
        if(outline.IsNull())throw std::runtime_error("Tool has no projected outline");
        BRepLib::BuildCurves3d(outline);
        gp_Trsf place;place.SetTransformation(gp_Ax3(plane.Location(),plane.Axis().Direction(),plane.XAxis().Direction()),gp_Ax3());
        BRepBuilderAPI_Transform transform(outline,place,true);
        Handle(TopTools_HSequenceOfShape) edges=new TopTools_HSequenceOfShape();
        Handle(TopTools_HSequenceOfShape) wires=new TopTools_HSequenceOfShape();
        for(TopExp_Explorer it(transform.Shape(),TopAbs_EDGE);it.More();it.Next())edges->Append(it.Current());
        ShapeAnalysis_FreeBounds::ConnectEdgesToWires(edges,1e-5,false,wires);
        BRepFeat_SplitShape op(shape);
        for(int i=1;i<=wires->Length();++i)op.Add(TopoDS::Wire(wires->Value(i)),face);
        if(!wires->Length())throw std::runtime_error("Tool outline does not cross the selected face");
        build_cancellable(op);if(!op.IsDone())throw std::runtime_error("Silhouette split failed");result=op.Shape();break;
      }
      case 13: {
        if (tool.IsNull()) throw std::runtime_error("Choose an alignment body");
        GProp_GProps a,b;BRepGProp::VolumeProperties(shape,a);BRepGProp::VolumeProperties(tool,b);
        gp_Trsf tr;tr.SetTranslation(gp_Vec(a.CentreOfMass(),b.CentreOfMass()));
        BRepBuilderAPI_Transform op(shape,tr,true);result=op.Shape();break;
      }
      case 14: case 15: {
        if (face.IsNull()) { consumed[e.target]=true;remember();continue; }
        result=heal_face(shape,face);break;
      }
      case 16: {
        if (!face.IsNull()) {
          result=heal_face(shape,face);
        } else {
          ShapeUpgrade_UnifySameDomain op(shape,true,true,true);op.Build();check_cancelled();result=op.Shape();
        }
        break;
      }
      default: throw std::runtime_error("Unknown solid edit");
      }
      if(e.kind==10 || e.kind==11) {
        unsigned before=0,after=0;
        for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next())++before;
        for(TopExp_Explorer it(result,TopAbs_FACE);it.More();it.Next())++after;
        if(after<=before)throw std::runtime_error("Splitting geometry does not subdivide the selected face");
      }
      if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid()) throw std::runtime_error("Edit produced invalid geometry");
      unsigned solids=0;for(TopExp_Explorer it(result,TopAbs_SOLID);it.More();it.Next())++solids;
      if (!solids) throw std::runtime_error("Edit removed all solid geometry");
      if (e.copy && (e.kind==5 || e.kind==12 || e.kind==13)) {
        TopoDS_Compound copies; BRep_Builder builder; builder.MakeCompound(copies);
        builder.Add(copies,shape);builder.Add(copies,result);outputs[e.target].shape=copies;outputs[e.target].caps.clear();
      } else { outputs[e.target].shape=result;outputs[e.target].caps.clear(); }
      remember();
    } catch(const Standard_Failure& error) {
      throw std::runtime_error("Solid edit "+std::to_string(n+1)+": "+error.GetMessageString());
    } catch(const std::exception& error) {
      throw std::runtime_error("Solid edit "+std::to_string(n+1)+": "+error.what());
    }
  }
}
