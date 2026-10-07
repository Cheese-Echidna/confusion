// Solid/Create algorithms share exact shapes with extrusion evaluation. All ownership
// remains in the worker-owned cache. Dimensions here are millimetres and radians.
static TopoDS_Shape moved(const TopoDS_Shape &s, const gp_Trsf &t) {
  BRepBuilderAPI_Transform op(s, t, true);
  if (!op.IsDone()) throw std::runtime_error("Could not transform body");
  return op.Shape();
}
static TopoDS_Shape translated(const TopoDS_Shape &s, double x, double y, double z) {
  gp_Trsf t; t.SetTranslation(gp_Vec(x,y,z)); return moved(s,t);
}
static TopoDS_Shape boolean_shape(const TopoDS_Shape &a, const TopoDS_Shape &b, int kind) {
  TopoDS_Shape result;
  if (kind == 0) { BRepAlgoAPI_Fuse op; boolean_inputs(op,a,b); build_cancellable(op); if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Join failed"); result = op.Shape(); }
  else if (kind == 1) { BRepAlgoAPI_Cut op; boolean_inputs(op,a,b); build_cancellable(op); if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Cut failed"); result = op.Shape(); }
  else { BRepAlgoAPI_Common op; boolean_inputs(op,a,b); build_cancellable(op); if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("Cell intersection failed"); result = op.Shape(); }
  check_solid(result);
  return result;
}
static TopoDS_Wire outer_wire(const TopoDS_Face &face) {
  TopExp_Explorer i(face, TopAbs_WIRE);
  if (!i.More()) throw std::runtime_error("Missing profile wire");
  auto wire = TopoDS::Wire(i.Current()); i.Next();
  if (i.More()) throw std::runtime_error("Loft sections currently require one outer wire without holes");
  return wire;
}
static TopoDS_Shape helix(double radius, double pitch, int turns, double section) {
  Handle(Geom_CylindricalSurface) surface = new Geom_CylindricalSurface(gp_Ax3(gp_Pnt(0,0,0),gp_Dir(0,0,1)),radius);
  Handle(Geom2d_Line) curve = new Geom2d_Line(gp_Pnt2d(0,0),gp_Dir2d(1,pitch/tau));
  BRepBuilderAPI_MakeEdge maker(curve,surface,0,turns*std::hypot(tau,pitch));
  if (!maker.IsDone()) throw std::runtime_error("Could not construct helix");
  auto edge = maker.Edge();
  if (!BRepLib::BuildCurve3d(edge,1e-6)) throw std::runtime_error("Could not approximate helix curve");
  BRepBuilderAPI_MakeWire spine(edge);
  gp_Circ circle(gp_Ax2(gp_Pnt(radius,0,0),gp_Dir(0,radius,pitch/tau)),section);
  BRepBuilderAPI_MakeWire wire(BRepBuilderAPI_MakeEdge(circle).Edge());
  BRepBuilderAPI_MakeFace face(wire.Wire());
  BRepOffsetAPI_MakePipe pipe(spine.Wire(),face.Face(),GeomFill_IsFrenet,true);
  if (!pipe.IsDone()) throw std::runtime_error("Helix sweep failed");
  check_solid(pipe.Shape()); return pipe.Shape();
}
static TopoDS_Shape gear(double module, int teeth, double height, double bore) {
  const double pitch = module*teeth/2., base = pitch*std::cos(20.*tau/360.);
  const double root = pitch-1.25*module, tip = pitch+module;
  auto involute = [&](double radius) { const double t = std::sqrt(std::max(0.,radius*radius/(base*base)-1.)); return t-std::atan(t); };
  const double half = tau/(4.*teeth), at_pitch = involute(pitch);
  BRepBuilderAPI_MakePolygon polygon;
  auto add = [&](double r, double a) { polygon.Add(gp_Pnt(r*std::cos(a),r*std::sin(a),0)); };
  for (int tooth=0; tooth<teeth; ++tooth) {
    check_cancelled();
    double a = tau*tooth/teeth;
    const double flank_root = std::max(root,base);
    add(root,a-half-at_pitch);
    for (int i=0;i<=8;++i) { double r=flank_root+(tip-flank_root)*i/8.; add(r,a-half-at_pitch+involute(r)); }
    for (int i=8;i>=0;--i) { double r=flank_root+(tip-flank_root)*i/8.; add(r,a+half+at_pitch-involute(r)); }
    add(root,a+half+at_pitch);
  }
  polygon.Close(); BRepBuilderAPI_MakeFace face(polygon.Wire());
  BRepPrimAPI_MakePrism prism(face.Face(),gp_Vec(0,0,height));
  auto result = prism.Shape(); check_solid(result);
  if (bore>0) result = boolean_shape(result,BRepPrimAPI_MakeCylinder(bore,height).Shape(),1);
  return result;
}
static void append_creates(rust::Slice<const ProfileEdge> edges,
                           rust::Slice<const CreateStep> creates,
                           std::vector<Output> &outputs, std::vector<bool> &consumed,
                           ModelCacheImpl *cache, rust::Slice<const rust::String> keys, size_t offset) {
  for (size_t index=0;index<creates.size();++index) {
    check_cancelled();
    try {
      const auto &s = creates[index];
      auto *entry = cache ? &cache->features[offset + index] : nullptr;
      if (entry && entry->key == std::string(keys[offset + index])) {
        for (int target : {s.target, s.second_target})
          if (target >= 0 && (static_cast<size_t>(target) >= consumed.size() || consumed[target]))
            throw std::runtime_error("Missing or consumed body");
        outputs.push_back(detached(entry->output));
        consumed.push_back(false);
        for (auto target : entry->consumed_targets) consumed.at(target) = true;
        ++cache->reused;
        continue;
      }
      const auto previous_consumed = consumed;
      for (double v : s.values) if (!std::isfinite(v) || std::abs(v)>1000.) throw std::runtime_error("Invalid Create value");
      auto value = [&](size_t i) { if (i>=s.values.size()) throw std::runtime_error("Missing Create value"); return s.values[i]; };
      auto mm = [&](size_t i) { return value(i)*1000.; };
      auto positive = [&](size_t i) { if (value(i)<=1e-7) throw std::runtime_error("Feature dimensions must be positive"); };
      auto count = [&](size_t i, int low, int high) { double v=value(i); if (v!=std::floor(v) || v<low || v>high) throw std::runtime_error("Invalid instance or turn count"); };
      if (s.kind==7 || s.kind==12) { count(2,1,32); positive(0); positive(1); positive(3); if (value(3)*2>=value(1) || value(3)>=value(0)) throw std::runtime_error("Helix section intersects adjacent turns or axis"); }
      if (s.kind==14) { count(0,1,32); count(1,1,32); if(value(0)*value(1)<2 || value(0)*value(1)>128) throw std::runtime_error("Pattern requires 2–128 instances"); }
      if (s.kind==15 || s.kind==16) count(0,2,128);
      if (s.kind==17) count(0,0,2);
      if (s.kind==20) { count(1,6,128); positive(0); positive(2); }

      auto body = [&](int i) -> const TopoDS_Shape & { if (i<0 || static_cast<size_t>(i)>=outputs.size() || consumed[i]) throw std::runtime_error("Missing or consumed body"); return outputs[i].shape; };
      auto profile = [&](uint32_t start, uint32_t count) { if (!count || start>edges.size() || count>edges.size()-start) throw std::runtime_error("Invalid Create profile range"); return profile_face(rust::Slice<const ProfileEdge>(edges.data()+start,count)); };
      TopoDS_Shape result;
      bool consume = false, consume_second = false;
      switch (s.kind) {
      case 0: { // revolve
        auto face = profile(s.edge_start,s.edge_count);
        BRepPrimAPI_MakeRevol revolve(face,gp_Ax1(gp_Pnt(mm(1),0,0),gp_Dir(0,1,0)),value(0));
        if (!revolve.IsDone()) throw std::runtime_error("Revolve failed");
        result=revolve.Shape(); break;
      }
      case 1: { // straight-vector sweep
        BRepPrimAPI_MakePrism sweep(profile(s.edge_start,s.edge_count),gp_Vec(mm(0),mm(1),mm(2)));
        if (!sweep.IsDone()) throw std::runtime_error("Sweep failed");
        result=sweep.Shape(); break;
      }
      case 2: {
        BRepOffsetAPI_ThruSections loft(true,false,1e-5);
        loft.AddWire(outer_wire(profile(s.edge_start,s.edge_count)));
        loft.AddWire(TopoDS::Wire(translated(outer_wire(profile(s.second_start,s.second_count)),0,0,mm(0))));
        build_cancellable(loft); if (!loft.IsDone()) throw std::runtime_error("Loft failed"); result=loft.Shape(); break;
      }
      case 3: case 4: { // positioned rectangular reinforcement
        auto tool=BRepPrimAPI_MakeBox(gp_Pnt(mm(3),mm(4),mm(5)),mm(0),mm(1),mm(2)).Shape();
        result=boolean_shape(body(s.target),tool,0); consume=true; break;
      }
      case 5: {
        auto face=translated(profile(s.edge_start,s.edge_count),0,0,mm(1));
        auto tool=BRepPrimAPI_MakePrism(face,gp_Vec(0,0,mm(0))).Shape();
        result=boolean_shape(body(s.target),tool,0); consume=true; break;
      }
      case 6: {
        auto tool=BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(mm(2),mm(3),mm(4)),gp_Dir(0,0,1)),mm(0),mm(1)).Shape();
        result=boolean_shape(body(s.target),tool,1); consume=true; break;
      }
      case 7: case 12: {
        auto tool=helix(mm(0),mm(1),static_cast<int>(value(2)),mm(3));
        result=s.kind==7 ? boolean_shape(body(s.target),tool,1) : tool; consume=s.kind==7; break;
      }
      case 8: result=BRepPrimAPI_MakeBox(mm(0),mm(1),mm(2)).Shape(); break;
      case 9: result=BRepPrimAPI_MakeCylinder(mm(0),mm(1)).Shape(); break;
      case 10: result=BRepPrimAPI_MakeSphere(mm(0)).Shape(); break;
      case 11: result=BRepPrimAPI_MakeTorus(mm(0),mm(1)).Shape(); break;
      case 13: result=boolean_shape(BRepPrimAPI_MakeCylinder(mm(0),mm(2)).Shape(),BRepPrimAPI_MakeCylinder(mm(0)-mm(1),mm(2)).Shape(),1); break;
      case 14: case 15: case 16: case 17: {
        auto source=body(s.target);
        TopoDS_Compound compound; BRep_Builder builder; builder.MakeCompound(compound);
        // The original stays a separate live body; this output owns only copies.
        if (s.kind==14) {
          int cols=static_cast<int>(value(0)), rows=static_cast<int>(value(1));
          for (int y=0;y<rows;++y) for (int x=0;x<cols;++x) {
            check_cancelled();
            if (x||y) builder.Add(compound,translated(source,x*mm(2),y*mm(3),0));
          }
        } else if (s.kind==15) {
          int count=static_cast<int>(value(0)); bool full=std::abs(value(1)-tau)<1e-8;
          for(int i=1;i<count;++i) { check_cancelled(); gp_Trsf t; t.SetRotation(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(0,0,1)),value(1)*i/(full?count:count-1)); builder.Add(compound,moved(source,t)); }
        } else if (s.kind==16) {
          int count=static_cast<int>(value(0)); for(int i=1;i<count;++i) { check_cancelled(); double f=static_cast<double>(i)/(count-1); builder.Add(compound,translated(source,mm(1)*f,mm(2)*f,mm(3)*f)); }
        } else {
          int plane=static_cast<int>(value(0)); gp_Pnt p(plane==0?mm(1):0,plane==1?mm(1):0,plane==2?mm(1):0);
          gp_Trsf t; t.SetMirror(gp_Ax2(p,gp_Dir(plane==0,plane==1,plane==2))); builder.Add(compound,moved(source,t));
        }
        result=compound; break;
      }
      case 18: {
        BRepOffsetAPI_MakeOffsetShape offset;
        run_cancellable([&](const Message_ProgressRange &range) {
          offset.PerformByJoin(body(s.target),mm(0),1e-4,BRepOffset_Skin,false,false,GeomAbs_Arc,true,range);
        });
        if (!offset.IsDone()) throw std::runtime_error("Body offset failed");
        result=boolean_shape(offset.Shape(),body(s.target),1); consume=true; break;
      }
      case 19: result=boolean_shape(body(s.target),body(s.second_target),2); consume=true; consume_second=true; break;
      case 20: result=gear(mm(0),static_cast<int>(value(1)),mm(2),mm(3)); break;
      default: throw std::runtime_error("Unknown Create operation");
      }
      if (s.kind<14 || s.kind>17) check_solid(result);
      else if (!BRepCheck_Analyzer(result).IsValid()) throw std::runtime_error("Invalid pattern result");
      if (consume) {
        GProp_GProps before, after; BRepGProp::VolumeProperties(body(s.target),before); BRepGProp::VolumeProperties(result,after);
        if (s.kind>=3 && s.kind<=7 && std::abs(before.Mass()-after.Mass())<1e-6) throw std::runtime_error("Feature does not change the target body");
        consumed[s.target]=true;
      }
      if (consume_second) consumed[s.second_target]=true;
      // Replaced topology intentionally has no extrusion cap anchors: unsupported
      // attachments fail instead of binding silently to unrelated faces.
      Output output{result,{},{}};
      name_generated(output.names,result,std::string(s.identity));
      outputs.push_back(std::move(output)); consumed.push_back(false);
      if (entry) {
        entry->key.clear();
        entry->output = detached(outputs.back());
        entry->consumed_targets.clear();
        for (size_t i = 0; i < previous_consumed.size(); ++i)
          if (!previous_consumed[i] && consumed[i]) entry->consumed_targets.push_back(i);
        entry->key = std::string(keys[offset + index]);
      }
    } catch(const Standard_Failure &e) { throw std::runtime_error("Create feature "+std::to_string(index+1)+": "+e.GetMessageString()); }
      catch(const std::exception &e) { throw std::runtime_error("Create feature "+std::to_string(index+1)+": "+e.what()); }
  }
}
