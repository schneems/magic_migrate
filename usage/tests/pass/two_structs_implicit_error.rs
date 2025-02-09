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
struct EmptyTitle;

impl From<std::convert::Infallible> for PersonMigrationError {
    fn from(_value: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl From<EmptyTitle> for PersonMigrationError {
    fn from(value: EmptyTitle) -> Self {
        PersonMigrationError::TitleCannotBeEmpty(value)
    }
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
enum PersonMigrationError {
    #[error("Title cannot be empty!!!")]
    TitleCannotBeEmpty(EmptyTitle),
}

impl TryFrom<PersonV1> for PersonV2 {
    type Error = EmptyTitle;

    fn try_from(value: PersonV1) -> Result<Self, Self::Error> {
        if let Some(title) = value.title {
            if title.is_empty() {
                Err(EmptyTitle)
            } else {
                Ok(PersonV2 {
                    name: value.name,
                    job_title: title,
                })
            }
        } else {
            Err(EmptyTitle)
        }
    }
}

fn main() {
    let _result: Result<PersonV2, PersonMigrationError> =
        PersonV2::try_from_str_migrations("name = 'richard'").unwrap();
}
