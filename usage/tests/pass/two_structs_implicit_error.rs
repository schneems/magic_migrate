use magic_migrate::TryMigrate;
use serde::Deserialize;

#[derive(Deserialize, Debug, TryMigrate)]
#[try_migrate(from = None, error = PersonMigrationError)]
struct PersonV1 {
    name: String,
    title: Option<String>,
}

#[derive(Deserialize, Debug, TryMigrate)]
#[try_migrate(from = PersonV1)]
struct PersonV2 {
    name: String,
    job_title: String,
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("Title cannot be empty")]
struct TitleCannotBeEmpty;

impl From<std::convert::Infallible> for PersonMigrationError {
    fn from(_value: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl From<TitleCannotBeEmpty> for PersonMigrationError {
    fn from(_value: TitleCannotBeEmpty) -> Self {
        PersonMigrationError::TitleCannotBeEmpty
    }
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
enum PersonMigrationError {
    #[error("Title cannot be empty!!!")]
    TitleCannotBeEmpty,
}

impl TryFrom<PersonV1> for PersonV2 {
    type Error = TitleCannotBeEmpty;

    fn try_from(value: PersonV1) -> Result<Self, TitleCannotBeEmpty> {
        if let Some(title) = value.title {
            if title.is_empty() {
                Err(TitleCannotBeEmpty)
            } else {
                Ok(PersonV2 {
                    name: value.name,
                    job_title: title,
                })
            }
        } else {
            Err(TitleCannotBeEmpty)
        }
    }
}

fn main() {
    let _result: Result<PersonV2, PersonMigrationError> =
        PersonV2::try_from_str_migrations("name = 'richard'").unwrap();
}
