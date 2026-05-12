use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Internal Server Error")]
    InternalServerError,
    
    #[error("Fetch User Error")]
    FetchUserError,
    
    #[error("Relational Not Found")]
    RelationalNotFound,

    #[error("Relational Document Not Found")]
    RelationalDocumentNotFound,

    #[error("Update User Error")]
    UpdateUserError,

    #[error("Relational Deserialize Error")]   
    RelationalDeserializeError
}