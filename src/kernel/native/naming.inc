// Semantic names track operation provenance, never evaluated face enumeration.
static void add_named(std::vector<NamedTopology>& names, const std::string& key,
                      const TopoDS_Shape& face, const TopoDS_Shape& result) {
  if (face.IsNull() || face.ShapeType()!=TopAbs_FACE || !has_face(result,face)) return;
  auto found=std::find_if(names.begin(),names.end(),[&](const auto& n){return n.key==key;});
  if(found==names.end()){names.push_back({key,{}});found=names.end()-1;}
  for(const auto& old:found->faces)if(old.IsSame(face))return;
  for(TopExp_Explorer it(result,TopAbs_FACE);it.More();it.Next())if(it.Current().IsSame(face)){found->faces.push_back(it.Current());break;}
}
template<class Operation>
static std::vector<NamedTopology> follow_names(Operation& op,
    const std::vector<NamedTopology>& input,const TopoDS_Shape& result) {
  std::vector<NamedTopology> names;
  for(const auto& source:input) {
    // Keep deleted names as empty records so resolution can explain the failure.
    names.push_back({source.key,{}});
    for(const auto& face:source.faces) {
      add_named(names,source.key,face,result);
      for(TopTools_ListIteratorOfListOfShape it(op.Modified(face));it.More();it.Next())add_named(names,source.key,it.Value(),result);
    }
  }
  return names;
}
static std::vector<NamedTopology> surviving_names(const std::vector<NamedTopology>& input,const TopoDS_Shape& result) {
  std::vector<NamedTopology> names;
  for(const auto& source:input){names.push_back({source.key,{}});for(const auto& face:source.faces)add_named(names,source.key,face,result);}return names;
}
// Newly generated faces are classified by generating feature and adjacency to
// already named faces. Identical classifications remain explicitly ambiguous.
static void name_generated(std::vector<NamedTopology>& names,const TopoDS_Shape& shape,const std::string& identity) {
  auto input=names;
  for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
    auto face=TopoDS::Face(it.Current());bool known=false;
    for(const auto& n:input)for(const auto& f:n.faces)if(f.IsSame(face))known=true;
    if(known)continue;
    BRepAdaptor_Surface surface(face);
    std::string key=identity+"/generated/"+std::to_string(int(surface.GetType()));
    if(surface.GetType()==GeomAbs_Plane) {
      auto n=surface.Plane().Axis().Direction();if(face.Orientation()==TopAbs_REVERSED)n.Reverse();
      // Surface orientation identifies primitive sides independent of dimensions.
      key+="/normal/"+std::to_string(std::lround(n.X()*1e6))+","+std::to_string(std::lround(n.Y()*1e6))+","+std::to_string(std::lround(n.Z()*1e6));
    }
    std::vector<std::string> adjacent;
    TopTools_IndexedMapOfShape edges;TopExp::MapShapes(face,TopAbs_EDGE,edges);
    for(const auto& name:input) {
      bool shared=false;
      for(const auto& f:name.faces)for(TopExp_Explorer e(f,TopAbs_EDGE);e.More();e.Next())if(edges.Contains(e.Current()))shared=true;
      if(shared)adjacent.push_back(name.key);
    }
    std::sort(adjacent.begin(),adjacent.end());adjacent.erase(std::unique(adjacent.begin(),adjacent.end()),adjacent.end());
    for(const auto& a:adjacent)key+="/adjacent/"+a;
    add_named(names,key,face,shape);
  }
}
static TopoDS_Face resolve_named(const std::vector<Output>& outputs,const std::vector<bool>& consumed,
    uint32_t target,const std::string& key) {
  if(target>=outputs.size()||consumed[target])throw std::runtime_error("Missing face reference body");
  std::vector<TopoDS_Shape> candidates;
  for(const auto& name:outputs[target].names)if(name.key==key)for(const auto& face:name.faces) {
    if(!has_face(outputs[target].shape,face))continue;
    bool duplicate=false;for(const auto& old:candidates)if(old.IsSame(face))duplicate=true;
    if(!duplicate)candidates.push_back(face);
  }
  if(candidates.empty())throw std::runtime_error("Referenced face was removed; reselect a face to repair this feature");
  if(candidates.size()!=1)throw std::runtime_error("Referenced face split or merged ambiguously; reselect a face to repair this feature");
  // A merged face may inherit multiple ancestors; using any one is ambiguous.
  unsigned ancestors=0;
  for(const auto& name:outputs[target].names)for(const auto& f:name.faces)if(f.IsSame(candidates[0])){++ancestors;break;}
  if(ancestors>1 && key.find("/branch/")==std::string::npos)throw std::runtime_error("Referenced face has merged ancestors; reselect a face to repair this feature");
  return TopoDS::Face(candidates[0]);
}
// A plane split supplies a semantic branch discriminator. Keep the old name
// ambiguous, and expose new branch names for explicit user repair.
static void name_split_branches(std::vector<NamedTopology>& names,const TopoDS_Shape& shape,
                                const ModifyStep& edit) {
  auto input=names;
  for(const auto& source:input)if(source.faces.size()>1) {
    for(const auto& face:source.faces) {
      GProp_GProps properties;BRepGProp::SurfaceProperties(face,properties);
      auto p=properties.CentreOfMass();gp_Vec normal(edit.b,edit.c,edit.d);normal.Normalize();
      double side=normal.Dot(gp_Vec(p.X(),p.Y(),p.Z()))-edit.a*1000;
      std::string branch=side>1e-6?"positive":side< -1e-6?"negative":"on_plane";
      add_named(names,source.key+"/branch/"+std::string(edit.identity)+"/"+branch,face,shape);
    }
  }
}
