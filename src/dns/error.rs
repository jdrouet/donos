// use crate::service::database::Error as DatabaseError;
use donos_proto::buffer::reader::ReaderError;
use donos_proto::buffer::writer::WriterError;
use std::fmt::Display;

#[derive(Debug)]
pub enum HandleError {
    Blocklist(Box<dyn std::error::Error>),
    Cache(std::io::Error),
    Lookup(std::io::Error),
    // Database(DatabaseError),
    Writer(WriterError),
    Reader(ReaderError),
    Io(std::io::Error),
    NoQuestion,
}

impl Display for HandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "handle error")
    }
}

// impl From<DatabaseError> for HandleError {
//     fn from(value: DatabaseError) -> Self {
//         Self::Database(value)
//     }
// }

impl From<WriterError> for HandleError {
    fn from(value: WriterError) -> Self {
        Self::Writer(value)
    }
}

impl From<ReaderError> for HandleError {
    fn from(value: ReaderError) -> Self {
        Self::Reader(value)
    }
}

impl From<std::io::Error> for HandleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
