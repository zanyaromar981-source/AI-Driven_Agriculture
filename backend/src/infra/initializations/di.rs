use std::sync::Arc;

use chrono::Duration;

use crate::{
    features::{
        farmers::{
            app::{
                FarmCounter, FarmerRepository, SignInChallengeRepository, SignInCodeGenerator,
                SignInCodeHasher, SignInCodeSender, TokenIssuer,
                use_cases::{
                    EditProfileUseCase, RequestSignInCodeUseCase, VerifySignInCodeUseCase,
                    ViewProfileUseCase,
                },
            },
            domain::SignInCode,
            infra::{
                FarmerPostgresRepository, FarmsFeatureFarmCounter, FixedSignInCodeGenerator,
                JwtTokenIssuer, LogSignInCodeSender, RandomSignInCodeGenerator,
                Sha256SignInCodeHasher, SignInChallengePostgresRepository,
            },
        },
        farms::{
            app::{
                FarmRepository,
                use_cases::{
                    ListFarmsUseCase, RegisterFarmUseCase, RemoveFarmUseCase,
                    RepaintFarmCellsUseCase, ViewFarmUseCase,
                },
            },
            infra::FarmPostgresRepository,
        },
        fires::{
            app::{
                FireRepository,
                use_cases::{ListFiresUseCase, RecordFireUseCase},
            },
            infra::FirePostgresRepository,
        },
        insights::{
            app::{
                FarmDirectory, FarmOwnership, InsightRepository,
                use_cases::{
                    ListFarmCoverageUseCase, RecordFarmInsightUseCase, ViewFarmInsightsUseCase,
                },
            },
            infra::{
                FarmsFeatureFarmDirectory, FarmsFeatureFarmOwnership, InsightPostgresRepository,
            },
        },
    },
    infra::{Config, DBConnector},
    shared::{FarmFeature, FarmerFeature, Features, FireFeature, InsightFeature},
};

pub async fn di_init(
    config: &Config,
    db_context: DBConnector,
) -> Result<Features, Box<dyn std::error::Error + Send + Sync>> {
    let farm_repository: Arc<dyn FarmRepository> =
        Arc::new(FarmPostgresRepository::new(db_context.conn_clone()));

    let farm = FarmFeature {
        register_farm_use_case: Arc::new(RegisterFarmUseCase::new(
            farm_repository.clone(),
            config.farm.max_farms_per_user,
            config.farm.max_cells_per_farm,
        )),
        list_farms_use_case: Arc::new(ListFarmsUseCase::new(farm_repository.clone())),
        view_farm_use_case: Arc::new(ViewFarmUseCase::new(farm_repository.clone())),
        repaint_farm_cells_use_case: Arc::new(RepaintFarmCellsUseCase::new(
            farm_repository.clone(),
        )),
        remove_farm_use_case: Arc::new(RemoveFarmUseCase::new(farm_repository.clone())),
    };

    let fire_repository: Arc<dyn FireRepository> =
        Arc::new(FirePostgresRepository::new(db_context.conn_clone()));

    let fire = FireFeature {
        list_fires_use_case: Arc::new(ListFiresUseCase::new(fire_repository.clone())),
        record_fire_use_case: Arc::new(RecordFireUseCase::new(fire_repository)),
    };

    let insight_repository: Arc<dyn InsightRepository> =
        Arc::new(InsightPostgresRepository::new(db_context.conn_clone()));
    let farm_ownership: Arc<dyn FarmOwnership> =
        Arc::new(FarmsFeatureFarmOwnership::new(farm_repository.clone()));
    let farm_directory: Arc<dyn FarmDirectory> =
        Arc::new(FarmsFeatureFarmDirectory::new(farm_repository.clone()));

    let insight = InsightFeature {
        view_farm_insights_use_case: Arc::new(ViewFarmInsightsUseCase::new(
            insight_repository.clone(),
            farm_ownership,
        )),
        record_farm_insight_use_case: Arc::new(RecordFarmInsightUseCase::new(
            insight_repository.clone(),
        )),
        list_farm_coverage_use_case: Arc::new(ListFarmCoverageUseCase::new(
            insight_repository,
            farm_directory,
        )),
    };

    let farmer_repository: Arc<dyn FarmerRepository> =
        Arc::new(FarmerPostgresRepository::new(db_context.conn_clone()));
    let challenge_repository: Arc<dyn SignInChallengeRepository> = Arc::new(
        SignInChallengePostgresRepository::new(db_context.conn_clone()),
    );
    let code_hasher: Arc<dyn SignInCodeHasher> =
        Arc::new(Sha256SignInCodeHasher::new(config.auth.jwt_secret.clone()));
    let code_sender: Arc<dyn SignInCodeSender> = Arc::new(LogSignInCodeSender);
    let token_issuer: Arc<dyn TokenIssuer> = Arc::new(JwtTokenIssuer::new(config.auth.clone()));
    let farm_counter: Arc<dyn FarmCounter> =
        Arc::new(FarmsFeatureFarmCounter::new(farm_repository));

    let code_generator: Arc<dyn SignInCodeGenerator> = match &config.auth.sign_in_code.fixed {
        Some(code) => {
            tracing::warn!(
                "AUTH__FIXED_SIGN_IN_CODE is set: every phone signs in with the same code"
            );

            Arc::new(FixedSignInCodeGenerator::new(SignInCode::new(
                code.clone(),
            )?))
        }
        None => Arc::new(RandomSignInCodeGenerator),
    };

    let farmer = FarmerFeature {
        request_sign_in_code_use_case: Arc::new(RequestSignInCodeUseCase::new(
            challenge_repository.clone(),
            code_generator,
            code_hasher.clone(),
            code_sender,
            Duration::minutes(config.auth.sign_in_code.valid_minutes),
            Duration::seconds(config.auth.sign_in_code.resend_after_seconds),
        )),
        verify_sign_in_code_use_case: Arc::new(VerifySignInCodeUseCase::new(
            farmer_repository.clone(),
            challenge_repository,
            code_hasher,
            token_issuer,
            farm_counter,
            config.auth.sign_in_code.max_attempts,
        )),
        view_profile_use_case: Arc::new(ViewProfileUseCase::new(farmer_repository.clone())),
        edit_profile_use_case: Arc::new(EditProfileUseCase::new(farmer_repository)),
    };

    Ok(Features {
        farm,
        farmer,
        fire,
        insight,
    })
}
