use diesel;
use diesel::mysql::MysqlConnection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;

use crate::errors::Error;
use crate::errors::ErrorKind;

use crate::tests::questions::models::{
    NewQuestion, Question, QuestionList, QuestionRequest, QuestionResponse,
};
use crate::tests::questions::schema::questions as questions_schema;

pub fn handle_question(
    request: QuestionRequest,
    database_connection: &MysqlConnection,
) -> Result<QuestionResponse, Error> {
    match request {
        QuestionRequest::GetQuestions => get_questions(database_connection)
            .map(|u| QuestionResponse::ManyQuestions(u)),
        QuestionRequest::CreateQuestion(question) => {
            create_question(question, database_connection)
                .map(|u| QuestionResponse::OneQuestion(u))
        }
        QuestionRequest::DeleteQuestion(id) => {
            delete_question(id, database_connection)
                .map(|_| QuestionResponse::NoResponse)
        }
    }
}

fn get_questions(
    database_connection: &MysqlConnection,
) -> Result<QuestionList, Error> {
    let found_questions =
        questions_schema::table.load::<Question>(database_connection)?;

    Ok(QuestionList {
        questions: found_questions,
    })
}

fn create_question(
    question: NewQuestion,
    database_connection: &MysqlConnection,
) -> Result<Question, Error> {
    diesel::insert_into(questions_schema::table)
        .values(question)
        .execute(database_connection)?;

    let mut inserted_questions = questions_schema::table
        .filter(diesel::dsl::sql("id = LAST_INSERT_ID()"))
        .load::<Question>(database_connection)?;

    if let Some(inserted_question) = inserted_questions.pop() {
        Ok(inserted_question)
    } else {
        Err(Error::new(ErrorKind::Database))
    }
}

fn delete_question(
    id: u64,
    database_connection: &MysqlConnection,
) -> Result<(), Error> {
    diesel::delete(questions_schema::table.filter(questions_schema::id.eq(id)))
        .execute(database_connection)?;

    Ok(())
}