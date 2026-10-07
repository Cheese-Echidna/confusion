// Borrowed token stays valid for the synchronous native call, including parallel
// OCCT tasks. Only the atomic Rust callback runs on those tasks; shapes stay local.
#pragma once
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressRange.hxx>
namespace confusion {
static thread_local const EvaluationCancellation *active_cancellation = nullptr;
class CancellationScope {
  const EvaluationCancellation *previous;
public:
  explicit CancellationScope(const EvaluationCancellation &token)
      : previous(active_cancellation) { active_cancellation = &token; }
  ~CancellationScope() { active_cancellation = previous; }
};
static void check_cancelled() {
  if (active_cancellation && evaluation_cancelled(*active_cancellation))
    throw std::runtime_error("Evaluation superseded");
}
class CancellationProgress : public Message_ProgressIndicator {
  const EvaluationCancellation *token;
public:
  explicit CancellationProgress(const EvaluationCancellation *value) : token(value) {}
protected:
  Standard_Boolean UserBreak() override {
    return token && evaluation_cancelled(*token);
  }
  void Show(const Message_ProgressScope &, Standard_Boolean) override {}
};
template<class Operation> static void run_cancellable(Operation operation) {
  check_cancelled();
  Handle(Message_ProgressIndicator) progress = new CancellationProgress(active_cancellation);
  operation(progress->Start());
  // Treat interruption as cancellation, rather than a failed geometry operation.
  check_cancelled();
}
template<class Algorithm> static void build_cancellable(Algorithm &algorithm) {
  run_cancellable([&](const Message_ProgressRange &range) { algorithm.Build(range); });
}
template<class Boolean> static void boolean_inputs(Boolean &operation,
    const TopoDS_Shape &first, const TopoDS_Shape &second) {
  TopTools_ListOfShape arguments, tools;
  arguments.Append(first); tools.Append(second);
  operation.SetArguments(arguments); operation.SetTools(tools);
}
}
