// use rocket::request::FromParam;
// use rocket::serde::{Deserializer, Serializer};
// use rocket::serde::{Deserialize, Serialize};

// use rocket_sync_db_pools::rusqlite;
// use rusqlite::types::FromSql;
// use rusqlite::ToSql;


#[macro_export]
macro_rules! make_id {
    ($id:ident) => {
        #[derive(Debug, Clone, Copy, std::hash::Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id(pub i64);

        impl rocket::serde::Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: rocket::serde::Serializer,
            {
                serializer.serialize_i64(self.0)
            }
        }
        
        impl<'de> rocket::serde::Deserialize<'de> for $id {
            fn deserialize<D>(deserializer: D) -> Result<$id, D::Error>
            where
                D: rocket::serde::Deserializer<'de>,
            {
                i64::deserialize(deserializer).map(Self)
            }
        }
        
        impl<'a> rocket::request::FromParam<'a> for $id{
            type Error = <i64 as rocket::request::FromParam<'a>>::Error;
        
            fn from_param(param: &'a str) -> std::prelude::v1::Result<Self, Self::Error> {
                i64::from_param(param).map(Self)
            }
        }
        
        impl rusqlite::ToSql for $id{
            fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                self.0.to_sql()
            }
        }
        
        impl rusqlite::types::FromSql for $id{
            fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
                i64::column_result(value).map(Self)
            }
        }
    };
}

