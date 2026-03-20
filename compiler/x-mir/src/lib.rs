// X 语言中层中间表示（MIR）
//
// MIR 是控制流图（CFG）形式的表示，适合进行：
// - 控制流分析和数据流分析
// - Perceus 内存管理分析
// - 优化 Pass（常量传播、DCE 等）
//
// 架构位置：HIR → MIR → LIR

pub mod lower;
pub mod mir;
pub mod perceus;

// 重导出主要类型
pub use lower::{lower_hir_to_mir, MirLowerError, MirLowerResult};
pub use mir::*;
pub use perceus::{
    analyze_hir, BasicBlock, ControlFlowAnalysis, FunctionAnalysis, FunctionSignature, InterproceduralContext,
    MemoryOp, OwnershipFact, OwnershipState, ParamOwnershipBehavior, PerceusAnalyzer, PerceusError,
    PerceusIR, ReturnOwnershipBehavior, ReuseAnalysis, ReusePair, SourcePos,
};
