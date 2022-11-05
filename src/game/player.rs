use crate::entities::player;
use crate::error::{Error, Result};
use crate::game::base::GameHandler;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};

impl GameHandler {
    async fn get_or_create_player(db: &DatabaseConnection, id: isize) -> Result<player::Model> {
        if let Some(player) = <player::Entity as EntityTrait>::find()
            .filter(
                Condition::all()
                    .add(<player::Entity as EntityTrait>::Column::TelegramId.eq(id as i32)),
            )
            .one(db)
            .await
            .map_err(|err| Error::DatabaseError(format!("Cannot fetch player. {}", err)))?
        {
            Ok(player)
        } else {
            Ok(player::ActiveModel {
                telegram_id: Set(id as i32),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|err| Error::DatabaseError(format!("Cannot insert player. {}", err)))?)
        }
    }

    async fn increase_player_score(
        db: &DatabaseConnection,
        player_id: isize,
        score: isize,
    ) -> Result<()> {
        let txn = db
            .begin()
            .await
            .map_err(|err| Error::DatabaseError(format!("Cannot begin transaction. {}", err)))?;
        let mut player = <player::Entity as EntityTrait>::find()
            .filter(
                Condition::all()
                    .add(<player::Entity as EntityTrait>::Column::TelegramId.eq(player_id as i32)),
            )
            .one(&txn)
            .await
            .map_err(|err| Error::DatabaseError(format!("Cannot fetch player. {}", err)))?
            .ok_or_else(|| Error::DatabaseError("Cannot find user model".to_string()))?;
        let old_score = player.score;
        let mut player_active_model: player::ActiveModel = player.into();
        player_active_model.score = Set(old_score + score as i32);
        player_active_model
            .update(&txn)
            .await
            .map_err(|err| Error::DatabaseError(format!("Cannot update user model. {}", err)))?;
        txn.commit()
            .await
            .map_err(|err| Error::DatabaseError(format!("Cannot commit transaction. {}", err)))?;
        Ok(())
    }
}
