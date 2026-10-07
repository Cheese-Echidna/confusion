// Exact STEP exchange. Native model coordinates are millimeters already.
#include <STEPControl_Writer.hxx>
#include <STEPControl_Reader.hxx>
#include <StepData_StepModel.hxx>
#include <DESTEP_Parameters.hxx>
namespace confusion {
Inspection write_step(const ModelCache &cache, rust::Str path) {
  try {
    const auto &shape = cache.impl->final_shape;
    if (shape.IsNull()) throw std::runtime_error("No evaluated solids to export");
    auto inspection = inspect_shape(shape);
    if (!inspection.valid || inspection.solids == 0)
      throw std::runtime_error("STEP export requires valid solids");
    STEPControl_Writer writer;
    writer.Model()->SetLocalLengthUnit(1.0);
    DESTEP_Parameters parameters;
    parameters.WriteUnit = UnitsMethods_LengthUnit_Millimeter;
    parameters.WriteSchema = DESTEP_Parameters::WriteMode_StepSchema_AP214IS;
    // Transfer each solid independently; compounds from patterns remain complete.
    for (TopExp_Explorer solid(shape, TopAbs_SOLID); solid.More(); solid.Next()) {
      if (writer.Transfer(solid.Current(), STEPControl_AsIs, parameters) != IFSelect_RetDone)
        throw std::runtime_error("STEP solid transfer failed");
    }
    if (writer.Write(std::string(path).c_str()) != IFSelect_RetDone)
      throw std::runtime_error("STEP file write failed");
    return inspection;
  } catch (const Standard_Failure &error) {
    throw std::runtime_error(error.GetMessageString());
  }
}
Mesh inspect_step(rust::Str path) {
  try {
    STEPControl_Reader reader;
    if (reader.ReadFile(std::string(path).c_str()) != IFSelect_RetDone)
      throw std::runtime_error("STEP file read failed");
    reader.SetSystemLengthUnit(1.0);
    if (reader.TransferRoots() == 0)
      throw std::runtime_error("STEP contains no transferable roots");
    auto shape = reader.OneShape();
    if (shape.IsNull()) throw std::runtime_error("STEP contains no shape");
    return mesh_shape(shape);
  } catch (const Standard_Failure &error) {
    throw std::runtime_error(error.GetMessageString());
  }
}
} // namespace confusion
