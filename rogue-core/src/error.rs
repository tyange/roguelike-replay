use std::fmt;

/// 데이터 파일을 불러오다 실패할 수 있는 세 가지 이유
#[derive(Debug)]
pub enum DataError {
    /// 파일을 못 읽었다. 원래 에러를 그대로 감싼다
    Io(std::io::Error),
    /// RON 문법이나 필드 이름이 틀렸다. 원래 에러를 그대로 감싼다
    Parse(ron::error::SpannedError),
    /// 문법은 맞지만 내용이 말이 안 된다
    Invalid { what: String, reason: String },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(_) => write!(f, "cannot read the data file"),
            DataError::Parse(_) => write!(f, "the data file has a syntax error"),
            DataError::Invalid { what, reason } => write!(f, "{what}: {reason}"),
        }
    }
}

impl std::error::Error for DataError {
    /// 원인 에러. 실행 파일은 이걸 따라가며 체인 전체를 보여 줄 수 있다
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataError::Io(e) => Some(e),
            DataError::Parse(e) => Some(e),
            DataError::Invalid { .. } => None,
        }
    }
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

impl From<ron::error::SpannedError> for DataError {
    fn from(e: ron::error::SpannedError) -> Self {
        DataError::Parse(e)
    }
}
