// Perceus内存管理库

#[derive(Debug, PartialEq, Clone)]
pub struct PerceusIR {
    // Perceus中间表示的根结构
}

/// 对高级中间表示进行Perceus分析
pub fn analyze_hir(_hir: &x_hir::Hir) -> Result<PerceusIR, PerceusError> {
    Ok(PerceusIR {})
}

/// Perceus分析错误
#[derive(thiserror::Error, Debug)]
pub enum PerceusError {
    #[error("分析错误: {0}")]
    AnalysisError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_hir_returns_ok_for_empty_hir() {
        let hir = x_hir::Hir {};
        let pir = analyze_hir(&hir).expect("analyze_hir");
        assert_eq!(pir, PerceusIR {});
    }

    #[test]
    fn perceus_error_displays_message() {
        let e = PerceusError::AnalysisError("test message".to_string());
        assert!(e.to_string().contains("分析错误"));
        assert!(e.to_string().contains("test message"));
    }
}
