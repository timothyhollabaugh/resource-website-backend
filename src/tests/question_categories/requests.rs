use diesel;
use diesel::mysql::MysqlConnection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;

use crate::errors::Error;
use crate::errors::ErrorKind;

use crate::tests::question_categories::models::{
    NewQuestionCategory, QuestionCategory, QuestionCategoryList, QuestionCategoryRequest, QuestionCategoryResponse,
};
use crate::tests::question_categories::schema::question_categories as question_categories_schema;

pub fn handle_question_category(
    request: QuestionCategoryRequest,
    database_connection: &MysqlConnection,
) -> Result<QuestionCategoryResponse, Error> {
    match request {
        QuestionCategoryRequest::GetQuestionCategorys => {
            get_question_categories(database_connection)
                .map(|u| QuestionCategoryResponse::ManyQuestionCategories(u))
        }
        QuestionCategoryRequest::CreateQuestionCategory(question_category) => {
            create_question_category(question_category, database_connection)
                .map(|u| QuestionCategoryResponse::OneQuestionCategory(u))
        }
        QuestionCategoryRequest::DeleteQuestionCategory(id) => {
            delete_question_category(id, database_connection)
                .map(|_| QuestionCategoryResponse::NoResponse)
        }
    }
}

fn get_question_categories(
    database_connection: &MysqlConnection,
) -> Result<QuestionCategoryList, Error> {
    let found_question_categories = question_categories_schema::table
        .load::<QuestionCategory>(database_connection)?;

    Ok(QuestionCategoryList {
        question_categories: found_question_categories,
    })
}

fn create_question_category(
    question_category: NewQuestionCategory,
    database_connection: &MysqlConnection,
) -> Result<QuestionCategory, Error> {
    diesel::insert_into(question_categories_schema::table)
        .values(question_category)
        .execute(database_connection)?;

    let mut inserted_question_categories = question_categories_schema::table
        .filter(diesel::dsl::sql("id = LAST_INSERT_ID()"))
        .load::<QuestionCategory>(database_connection)?;

    if let Some(inserted_question_category) = inserted_question_categories.pop()
    {
        Ok(inserted_question_category)
    } else {
        Err(Error::new(ErrorKind::Database))
    }
}

fn delete_question_category(
    id: u64,
    database_connection: &MysqlConnection,
) -> Result<(), Error> {
    diesel::delete(
        question_categories_schema::table
            .filter(question_categories_schema::id.eq(id)),
    )
    .execute(database_connection)?;

    Ok(())
}