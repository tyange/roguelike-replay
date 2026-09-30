use std::fmt;

/// 데이터 파일이나 저장 파일을 다루다 실패할 수 있는 이유
#[derive(Debug)]
pub enum DataError {
    /// 파일을 못 읽었다. 원래 에러를 그대로 감싼다
    Io(std::io::Error),
    /// RON 문법이나 필드 이름이 틀렸다. 원래 에러를 그대로 감싼다
    Parse(ron::error::SpannedError),
    /// 문법은 맞지만 내용이 말이 안 된다
    Invalid { what: String, reason: String },
    /// 저장할 때 인코딩에 실패했다
    Encode(ron::Error),
    /// 저장 파일의 버전이 이 프로그램과 다르다
    Version { found: u32, expected: u32 },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(_) => write!(f, "cannot read the data file"),
            DataError::Parse(_) => write!(f, "the data file has a syntax error"),
            DataError::Invalid { what, reason } => write!(f, "{what}: {reason}"),
            DataError::Encode(_) => write!(f, "cannot encode the save file"),
            DataError::Version { found, expected } => {
                write!(
                    f,
                    "save file version {found} is not supported (expected {expected})"
                )
            }
        }
    }
}

impl std::error::Error for DataError {
    /// 원인 에러. 실행 파일은 이걸 따라가며 체인 전체를 보여 줄 수 있다
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataError::Io(e) => Some(e),
            DataError::Parse(e) => Some(e),
            DataError::Encode(e) => Some(e),
            DataError::Invalid { .. } | DataError::Version { .. } => None,
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

impl From<ron::Error> for DataError {
    fn from(e: ron::Error) -> Self {
        DataError::Encode(e)
    }
}
