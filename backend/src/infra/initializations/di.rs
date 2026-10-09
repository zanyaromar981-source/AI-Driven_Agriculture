use std::sync::Arc;

use chrono::Duration;

use crate::{
    features::{
        alwa::{
            app::{
                AlwaRepository,
                use_cases::{
                    AcceptOfferUseCase, BrowseListingsUseCase, CancelListingUseCase,
                    ListDealsUseCase, ListMarketsUseCase, ListMyListingsUseCase,
                    ListMyOffersUseCase, MakeOfferUseCase, PostListingUseCase, RecordPriceUseCase,
                    ViewListingUseCase, ViewMarketPricesUseCase, ViewPriceHistoryUseCase,
                },
            },
            domain::MAX_OPEN_LISTINGS_PER_SELLER,
            infra::AlwaPostgresRepository,
        },
        dams::{
            app::{
                DamRepository,
                use_cases::{ListDamsUseCase, RecordDamReadingUseCase, ViewDamHistoryUseCase},
            },
            infra::DamPostgresRepository,
        },
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
                use_cases::{
                    CorrectFireUseCase, CreateFireUseCase, ListFiresUseCase,
                    ListStoredFiresUseCase, RecordFireUseCase, RemoveFireUseCase,
                    ViewStoredFireUseCase,
                },
            },
            infra::FirePostgresRepository,
        },
        insights::{
            app::{
                FarmDirectory, FarmOwnership, InsightRepository,
                use_cases::{
                    CorrectFarmInsightUseCase, CreateFarmInsightUseCase, ListFarmCoverageUseCase,
                    RecordFarmInsightUseCase, RemoveFarmInsightUseCase, ViewFarmInsightsUseCase,
                    ViewStoredFarmInsightsUseCase,
                },
            },
            infra::{
                FarmsFeatureFarmDirectory, FarmsFeatureFarmOwnership, InsightPostgresRepository,
            },
        },
        outlooks::{
            app::{
                OutlookRepository,
                use_cases::{
                    RecordOutlookRunUseCase, RecordZoneOutlookUseCase, ViewSeasonOutlookUseCase,
                    ViewZoneOutlookUseCase,
                },
            },
            infra::OutlookPostgresRepository,
        },
        staff::{
            app::{
                PasswordHasher, RoleRepository, StaffRepository, StaffTokenIssuer,
                use_cases::{
                    AddStaffUseCase, CreateOwnerUseCase, CreateRoleUseCase, DeleteRoleUseCase,
                    EditRoleUseCase, EditStaffUseCase, IdentifyStaffUseCase, ListRolesUseCase,
                    ListStaffUseCase, RemoveStaffUseCase, SignInUseCase, ViewRoleUseCase,
                    ViewStaffUseCase,
                },
            },
            infra::{
                Argon2idPasswordHasher, JwtStaffTokenIssuer, RolePostgresRepository,
                StaffPostgresRepository,
            },
        },
        water::{
            app::{
                WaterPlanRepository,
                use_cases::{
                    RemoveWaterPlanEntryUseCase, SetWaterPlanEntryUseCase, ViewWaterPlanUseCase,
                },
            },
            infra::WaterPlanPostgresRepository,
        },
        zones::{
            app::{
                ZoneRepository,
                use_cases::{
                    CompareYearsUseCase, RecordSubZoneReadingUseCase, RecordZoneReadingUseCase,
                    ViewRegionOverviewUseCase, ViewZoneUseCase,
                },
            },
            infra::ZonePostgresRepository,
        },
    },
    infra::{Config, DBConnector},
    shared::{
        AlwaFeature, DamFeature, FarmFeature, FarmerFeature, Features, FireFeature, InsightFeature,
        OutlookFeature, StaffFeature, WaterFeature, ZoneFeature,
    },
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
        record_fire_use_case: Arc::new(RecordFireUseCase::new(fire_repository.clone())),
        list_stored_fires_use_case: Arc::new(ListStoredFiresUseCase::new(fire_repository.clone())),
        view_stored_fire_use_case: Arc::new(ViewStoredFireUseCase::new(fire_repository.clone())),
        create_fire_use_case: Arc::new(CreateFireUseCase::new(fire_repository.clone())),
        correct_fire_use_case: Arc::new(CorrectFireUseCase::new(fire_repository.clone())),
        remove_fire_use_case: Arc::new(RemoveFireUseCase::new(fire_repository)),
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
            insight_repository.clone(),
            farm_directory.clone(),
        )),
        view_stored_farm_insights_use_case: Arc::new(ViewStoredFarmInsightsUseCase::new(
            insight_repository.clone(),
            farm_directory.clone(),
        )),
        create_farm_insight_use_case: Arc::new(CreateFarmInsightUseCase::new(
            insight_repository.clone(),
            farm_directory,
        )),
        correct_farm_insight_use_case: Arc::new(CorrectFarmInsightUseCase::new(
            insight_repository.clone(),
        )),
        remove_farm_insight_use_case: Arc::new(RemoveFarmInsightUseCase::new(insight_repository)),
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
            Duration::seconds(config.auth.sign_in_code.reuse_window_seconds),
        )),
        view_profile_use_case: Arc::new(ViewProfileUseCase::new(farmer_repository.clone())),
        edit_profile_use_case: Arc::new(EditProfileUseCase::new(farmer_repository)),
    };

    let zone_repository: Arc<dyn ZoneRepository> =
        Arc::new(ZonePostgresRepository::new(db_context.conn_clone()));

    let zone = ZoneFeature {
        view_region_overview_use_case: Arc::new(ViewRegionOverviewUseCase::new(
            zone_repository.clone(),
        )),
        view_zone_use_case: Arc::new(ViewZoneUseCase::new(zone_repository.clone())),
        compare_years_use_case: Arc::new(CompareYearsUseCase::new(zone_repository.clone())),
        record_zone_reading_use_case: Arc::new(RecordZoneReadingUseCase::new(
            zone_repository.clone(),
        )),
        record_sub_zone_reading_use_case: Arc::new(RecordSubZoneReadingUseCase::new(
            zone_repository,
        )),
    };

    let dam_repository: Arc<dyn DamRepository> =
        Arc::new(DamPostgresRepository::new(db_context.conn_clone()));

    let dam = DamFeature {
        list_dams_use_case: Arc::new(ListDamsUseCase::new(dam_repository.clone())),
        view_dam_history_use_case: Arc::new(ViewDamHistoryUseCase::new(dam_repository.clone())),
        record_dam_reading_use_case: Arc::new(RecordDamReadingUseCase::new(dam_repository)),
    };

    let outlook_repository: Arc<dyn OutlookRepository> =
        Arc::new(OutlookPostgresRepository::new(db_context.conn_clone()));

    let outlook = OutlookFeature {
        view_season_outlook_use_case: Arc::new(ViewSeasonOutlookUseCase::new(
            outlook_repository.clone(),
        )),
        view_zone_outlook_use_case: Arc::new(ViewZoneOutlookUseCase::new(
            outlook_repository.clone(),
        )),
        record_zone_outlook_use_case: Arc::new(RecordZoneOutlookUseCase::new(
            outlook_repository.clone(),
        )),
        record_outlook_run_use_case: Arc::new(RecordOutlookRunUseCase::new(outlook_repository)),
    };

    let water_plan_repository: Arc<dyn WaterPlanRepository> =
        Arc::new(WaterPlanPostgresRepository::new(db_context.conn_clone()));

    let water = WaterFeature {
        view_water_plan_use_case: Arc::new(ViewWaterPlanUseCase::new(
            water_plan_repository.clone(),
        )),
        set_water_plan_entry_use_case: Arc::new(SetWaterPlanEntryUseCase::new(
            water_plan_repository.clone(),
        )),
        remove_water_plan_entry_use_case: Arc::new(RemoveWaterPlanEntryUseCase::new(
            water_plan_repository,
        )),
    };

    let alwa_repository: Arc<dyn AlwaRepository> =
        Arc::new(AlwaPostgresRepository::new(db_context.conn_clone()));

    let alwa = AlwaFeature {
        list_markets_use_case: Arc::new(ListMarketsUseCase::new(alwa_repository.clone())),
        view_market_prices_use_case: Arc::new(ViewMarketPricesUseCase::new(
            alwa_repository.clone(),
        )),
        view_price_history_use_case: Arc::new(ViewPriceHistoryUseCase::new(
            alwa_repository.clone(),
        )),
        record_price_use_case: Arc::new(RecordPriceUseCase::new(alwa_repository.clone())),
        browse_listings_use_case: Arc::new(BrowseListingsUseCase::new(alwa_repository.clone())),
        view_listing_use_case: Arc::new(ViewListingUseCase::new(alwa_repository.clone())),
        list_deals_use_case: Arc::new(ListDealsUseCase::new(alwa_repository.clone())),
        post_listing_use_case: Arc::new(PostListingUseCase::new(
            alwa_repository.clone(),
            MAX_OPEN_LISTINGS_PER_SELLER,
        )),
        list_my_listings_use_case: Arc::new(ListMyListingsUseCase::new(alwa_repository.clone())),
        cancel_listing_use_case: Arc::new(CancelListingUseCase::new(alwa_repository.clone())),
        make_offer_use_case: Arc::new(MakeOfferUseCase::new(alwa_repository.clone())),
        accept_offer_use_case: Arc::new(AcceptOfferUseCase::new(alwa_repository.clone())),
        list_my_offers_use_case: Arc::new(ListMyOffersUseCase::new(alwa_repository)),
    };

    let role_repository: Arc<dyn RoleRepository> =
        Arc::new(RolePostgresRepository::new(db_context.conn_clone()));
    let staff_repository: Arc<dyn StaffRepository> =
        Arc::new(StaffPostgresRepository::new(db_context.conn_clone()));
    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2idPasswordHasher::new()?);
    let staff_token_issuer: Arc<dyn StaffTokenIssuer> =
        Arc::new(JwtStaffTokenIssuer::new(config.auth.clone()));

    let staff = StaffFeature {
        sign_in_use_case: Arc::new(SignInUseCase::new(
            staff_repository.clone(),
            password_hasher.clone(),
            staff_token_issuer,
        )),
        identify_staff_use_case: Arc::new(IdentifyStaffUseCase::new(staff_repository.clone())),
        list_roles_use_case: Arc::new(ListRolesUseCase::new(role_repository.clone())),
        view_role_use_case: Arc::new(ViewRoleUseCase::new(role_repository.clone())),
        create_role_use_case: Arc::new(CreateRoleUseCase::new(role_repository.clone())),
        edit_role_use_case: Arc::new(EditRoleUseCase::new(role_repository.clone())),
        delete_role_use_case: Arc::new(DeleteRoleUseCase::new(role_repository)),
        list_staff_use_case: Arc::new(ListStaffUseCase::new(staff_repository.clone())),
        view_staff_use_case: Arc::new(ViewStaffUseCase::new(staff_repository.clone())),
        add_staff_use_case: Arc::new(AddStaffUseCase::new(
            staff_repository.clone(),
            password_hasher.clone(),
        )),
        edit_staff_use_case: Arc::new(EditStaffUseCase::new(
            staff_repository.clone(),
            password_hasher.clone(),
        )),
        remove_staff_use_case: Arc::new(RemoveStaffUseCase::new(staff_repository.clone())),
        create_owner_use_case: Arc::new(CreateOwnerUseCase::new(staff_repository, password_hasher)),
    };

    Ok(Features {
        farm,
        farmer,
        fire,
        insight,
        zone,
        dam,
        outlook,
        water,
        alwa,
        staff,
    })
}
