use std::sync::Arc;

use chrono::Duration;

use crate::features::alwa::app::use_cases::{
    CreateMarketUseCase, CreatePriceUseCase, DeleteListingUseCase, DeleteMarketUseCase,
    DeletePriceUseCase, ListAllListingsUseCase, ListStoredPricesUseCase, ModerateListingUseCase,
    UpdateMarketUseCase, UpdatePriceUseCase,
};
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
        briefs::{
            app::{
                BriefFarmOwnership, BriefRepository,
                use_cases::{
                    CorrectBriefUseCase, DeleteBriefUseCase, ListBriefsUseCase,
                    ListStoredBriefsUseCase, RecordBriefUseCase, RecordFarmZonesUseCase,
                    RemoveBriefUseCase, ViewFarmBriefUseCase, ViewLatestBriefUseCase,
                },
            },
            infra::{BriefPostgresRepository, FarmsFeatureBriefFarmOwnership},
        },
        dams::{
            app::{
                DamRepository,
                use_cases::{
                    CreateDamReadingUseCase, DeleteDamReadingUseCase, ListDamReadingsUseCase,
                    ListDamsUseCase, ListReferenceDamsUseCase, RecordDamReadingUseCase,
                    UpdateDamReadingUseCase, ViewDamHistoryUseCase,
                },
            },
            infra::DamPostgresRepository,
        },
        doctor::{
            app::{Doctor, FarmBriefs, FarmHistory, use_cases::AskDoctorUseCase},
            infra::{FarmsFeatureFarmBriefs, HttpDoctor, InsightsFeatureFarmHistory},
        },
        farmers::{
            app::{
                FarmCounter, FarmRemover, FarmerRepository, SignInChallengeRepository,
                SignInCodeGenerator, SignInCodeHasher, SignInCodeSender, TokenIssuer,
                use_cases::{
                    EditFarmerUseCase, EditProfileUseCase, EnsureFarmerUseCase,
                    IdentifyFarmerUseCase, ListFarmersUseCase, RegisterFarmerUseCase,
                    RemoveFarmerUseCase, RequestSignInCodeUseCase, VerifySignInCodeUseCase,
                    ViewFarmerUseCase, ViewProfileUseCase,
                },
            },
            domain::SignInCode,
            infra::{
                FarmerPostgresRepository, FarmsFeatureFarmCounter, FarmsFeatureFarmRemover,
                FixedSignInCodeGenerator, JwtTokenIssuer, LogSignInCodeSender,
                OtpiqSignInCodeSender, RandomSignInCodeGenerator, Sha256SignInCodeHasher,
                SignInChallengePostgresRepository,
            },
        },
        farms::{
            app::{
                FarmRepository, FarmerDirectory,
                use_cases::{
                    EditFarmUseCase, ListAllFarmsUseCase, ListFarmsUseCase,
                    RegisterFarmForFarmerUseCase, RegisterFarmUseCase, RemoveAnyFarmUseCase,
                    RemoveFarmUseCase, RenameFarmUseCase, RepaintFarmCellsUseCase,
                    ViewAnyFarmUseCase, ViewFarmUseCase,
                },
            },
            infra::{FarmPostgresRepository, FarmersFeatureFarmerDirectory},
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
                    CreateOutlookRunUseCase, CreateZoneOutlookUseCase, DeleteOutlookRunUseCase,
                    DeleteZoneOutlookUseCase, ListOutlookRunsUseCase, ListZoneOutlooksUseCase,
                    RecordOutlookRunUseCase, RecordZoneOutlookUseCase, UpdateOutlookRunUseCase,
                    UpdateZoneOutlookUseCase, ViewSeasonOutlookUseCase, ViewZoneOutlookUseCase,
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
                    CreateWaterPlanEntryUseCase, DeleteWaterPlanEntryUseCase,
                    ListWaterPlanEntriesUseCase, ListWaterSeasonsUseCase,
                    RemoveWaterPlanEntryUseCase, SetWaterPlanEntryUseCase,
                    UpdateWaterPlanEntryUseCase, ViewWaterPlanUseCase,
                },
            },
            infra::WaterPlanPostgresRepository,
        },
        zones::{
            app::{
                ZoneRepository,
                use_cases::{
                    CompareYearsUseCase, CreateSubZoneReadingUseCase, CreateZoneReadingUseCase,
                    DeleteSubZoneReadingUseCase, DeleteZoneReadingUseCase,
                    ListSubZoneReadingsUseCase, ListZoneReadingsUseCase, ListZonesUseCase,
                    RecordSubZoneReadingUseCase, RecordZoneReadingUseCase,
                    UpdateSubZoneReadingUseCase, UpdateZoneReadingUseCase,
                    ViewRegionOverviewUseCase, ViewZoneUseCase,
                },
            },
            infra::ZonePostgresRepository,
        },
    },
    infra::{Config, DBConnector},
    shared::{
        AlwaFeature, BriefFeature, DamFeature, DoctorFeature, FarmFeature, FarmerFeature, Features,
        FireFeature, InsightFeature, OutlookFeature, StaffFeature, WaterFeature, ZoneFeature,
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
        edit_farm_use_case: Arc::new(EditFarmUseCase::new(
            farm_repository.clone(),
            config.farm.max_cells_per_farm,
        )),
        remove_farm_use_case: Arc::new(RemoveFarmUseCase::new(farm_repository.clone())),
        list_all_farms_use_case: Arc::new(ListAllFarmsUseCase::new(farm_repository.clone())),
        view_any_farm_use_case: Arc::new(ViewAnyFarmUseCase::new(farm_repository.clone())),
        register_farm_for_farmer_use_case: Arc::new(RegisterFarmForFarmerUseCase::new(
            Arc::new(FarmersFeatureFarmerDirectory::new(Arc::new(
                FarmerPostgresRepository::new(db_context.conn_clone()),
            ))) as Arc<dyn FarmerDirectory>,
            Arc::new(RegisterFarmUseCase::new(
                farm_repository.clone(),
                config.farm.max_farms_per_user,
                config.farm.max_cells_per_farm,
            )),
        )),
        rename_farm_use_case: Arc::new(RenameFarmUseCase::new(farm_repository.clone())),
        remove_any_farm_use_case: Arc::new(RemoveAnyFarmUseCase::new(farm_repository.clone())),
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
    let farm_briefs: Arc<dyn FarmBriefs> =
        Arc::new(FarmsFeatureFarmBriefs::new(farm_repository.clone()));
    let farm_history: Arc<dyn FarmHistory> =
        Arc::new(InsightsFeatureFarmHistory::new(insight_repository.clone()));

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

    let brief_repository: Arc<dyn BriefRepository> =
        Arc::new(BriefPostgresRepository::new(db_context.conn_clone()));
    let brief_farm_ownership: Arc<dyn BriefFarmOwnership> =
        Arc::new(FarmsFeatureBriefFarmOwnership::new(farm_repository.clone()));

    let brief = BriefFeature {
        view_latest_brief_use_case: Arc::new(ViewLatestBriefUseCase::new(brief_repository.clone())),
        list_briefs_use_case: Arc::new(ListBriefsUseCase::new(brief_repository.clone())),
        view_farm_brief_use_case: Arc::new(ViewFarmBriefUseCase::new(
            brief_repository.clone(),
            brief_farm_ownership,
        )),
        record_brief_use_case: Arc::new(RecordBriefUseCase::new(brief_repository.clone())),
        record_farm_zones_use_case: Arc::new(RecordFarmZonesUseCase::new(brief_repository.clone())),
        delete_brief_use_case: Arc::new(DeleteBriefUseCase::new(brief_repository.clone())),
        list_stored_briefs_use_case: Arc::new(ListStoredBriefsUseCase::new(
            brief_repository.clone(),
        )),
        correct_brief_use_case: Arc::new(CorrectBriefUseCase::new(brief_repository.clone())),
        remove_brief_use_case: Arc::new(RemoveBriefUseCase::new(brief_repository)),
    };

    let farmer_repository: Arc<dyn FarmerRepository> =
        Arc::new(FarmerPostgresRepository::new(db_context.conn_clone()));
    let challenge_repository: Arc<dyn SignInChallengeRepository> = Arc::new(
        SignInChallengePostgresRepository::new(db_context.conn_clone()),
    );
    let code_hasher: Arc<dyn SignInCodeHasher> =
        Arc::new(Sha256SignInCodeHasher::new(config.auth.jwt_secret.clone()));
    let code_sender: Arc<dyn SignInCodeSender> = match &config.otpiq.api_key {
        Some(api_key) => {
            if config.auth.sign_in_code.fixed.is_some() {
                return Err(
                    "AUTH__FIXED_SIGN_IN_CODE and OTPIQ__API_KEY are both set: a fixed code \
                     with real delivery would text every phone the same code. Unset one of them."
                        .into(),
                );
            }

            let sender = OtpiqSignInCodeSender::new(api_key.clone(), &config.otpiq)?;

            tracing::info!(
                sender = "otpiq",
                provider = %config.otpiq.provider,
                base_url = %config.otpiq.base_url,
                timeout_seconds = config.otpiq.timeout_seconds,
                "sign-in codes are delivered through OTPIQ"
            );

            Arc::new(sender)
        }
        None => {
            tracing::warn!(
                sender = "log",
                "OTPIQ__API_KEY is not set: sign-in codes are written to the server log"
            );

            Arc::new(LogSignInCodeSender)
        }
    };
    let token_issuer: Arc<dyn TokenIssuer> = Arc::new(JwtTokenIssuer::new(config.auth.clone()));
    let dashboard_farm_counter: Arc<dyn FarmCounter> =
        Arc::new(FarmsFeatureFarmCounter::new(farm_repository.clone()));
    let dashboard_farm_remover: Arc<dyn FarmRemover> =
        Arc::new(FarmsFeatureFarmRemover::new(farm_repository.clone()));
    let dashboard_farmer_repository = farmer_repository.clone();
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
        identify_farmer_use_case: Arc::new(IdentifyFarmerUseCase::new(farmer_repository.clone())),
        ensure_farmer_use_case: Arc::new(EnsureFarmerUseCase::new(farmer_repository.clone())),
        view_profile_use_case: Arc::new(ViewProfileUseCase::new(farmer_repository.clone())),
        edit_profile_use_case: Arc::new(EditProfileUseCase::new(farmer_repository)),
        list_farmers_use_case: Arc::new(ListFarmersUseCase::new(
            dashboard_farmer_repository.clone(),
            dashboard_farm_counter.clone(),
        )),
        view_farmer_use_case: Arc::new(ViewFarmerUseCase::new(
            dashboard_farmer_repository.clone(),
            dashboard_farm_counter.clone(),
        )),
        register_farmer_use_case: Arc::new(RegisterFarmerUseCase::new(
            dashboard_farmer_repository.clone(),
            dashboard_farm_counter.clone(),
        )),
        edit_farmer_use_case: Arc::new(EditFarmerUseCase::new(
            dashboard_farmer_repository.clone(),
            dashboard_farm_counter,
        )),
        remove_farmer_use_case: Arc::new(RemoveFarmerUseCase::new(
            dashboard_farmer_repository,
            dashboard_farm_remover,
        )),
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
            zone_repository.clone(),
        )),
        list_zones_use_case: Arc::new(ListZonesUseCase::new(zone_repository.clone())),
        list_zone_readings_use_case: Arc::new(ListZoneReadingsUseCase::new(
            zone_repository.clone(),
        )),
        create_zone_reading_use_case: Arc::new(CreateZoneReadingUseCase::new(
            zone_repository.clone(),
        )),
        update_zone_reading_use_case: Arc::new(UpdateZoneReadingUseCase::new(
            zone_repository.clone(),
        )),
        delete_zone_reading_use_case: Arc::new(DeleteZoneReadingUseCase::new(
            zone_repository.clone(),
        )),
        list_sub_zone_readings_use_case: Arc::new(ListSubZoneReadingsUseCase::new(
            zone_repository.clone(),
        )),
        create_sub_zone_reading_use_case: Arc::new(CreateSubZoneReadingUseCase::new(
            zone_repository.clone(),
        )),
        update_sub_zone_reading_use_case: Arc::new(UpdateSubZoneReadingUseCase::new(
            zone_repository.clone(),
        )),
        delete_sub_zone_reading_use_case: Arc::new(DeleteSubZoneReadingUseCase::new(
            zone_repository,
        )),
    };

    let dam_repository: Arc<dyn DamRepository> =
        Arc::new(DamPostgresRepository::new(db_context.conn_clone()));

    let dam = DamFeature {
        list_dams_use_case: Arc::new(ListDamsUseCase::new(dam_repository.clone())),
        view_dam_history_use_case: Arc::new(ViewDamHistoryUseCase::new(dam_repository.clone())),
        record_dam_reading_use_case: Arc::new(RecordDamReadingUseCase::new(dam_repository.clone())),
        list_reference_dams_use_case: Arc::new(ListReferenceDamsUseCase::new(
            dam_repository.clone(),
        )),
        list_dam_readings_use_case: Arc::new(ListDamReadingsUseCase::new(dam_repository.clone())),
        create_dam_reading_use_case: Arc::new(CreateDamReadingUseCase::new(dam_repository.clone())),
        update_dam_reading_use_case: Arc::new(UpdateDamReadingUseCase::new(dam_repository.clone())),
        delete_dam_reading_use_case: Arc::new(DeleteDamReadingUseCase::new(dam_repository)),
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
        record_outlook_run_use_case: Arc::new(RecordOutlookRunUseCase::new(
            outlook_repository.clone(),
        )),
        list_zone_outlooks_use_case: Arc::new(ListZoneOutlooksUseCase::new(
            outlook_repository.clone(),
        )),
        create_zone_outlook_use_case: Arc::new(CreateZoneOutlookUseCase::new(
            outlook_repository.clone(),
        )),
        update_zone_outlook_use_case: Arc::new(UpdateZoneOutlookUseCase::new(
            outlook_repository.clone(),
        )),
        delete_zone_outlook_use_case: Arc::new(DeleteZoneOutlookUseCase::new(
            outlook_repository.clone(),
        )),
        list_outlook_runs_use_case: Arc::new(ListOutlookRunsUseCase::new(
            outlook_repository.clone(),
        )),
        create_outlook_run_use_case: Arc::new(CreateOutlookRunUseCase::new(
            outlook_repository.clone(),
        )),
        update_outlook_run_use_case: Arc::new(UpdateOutlookRunUseCase::new(
            outlook_repository.clone(),
        )),
        delete_outlook_run_use_case: Arc::new(DeleteOutlookRunUseCase::new(outlook_repository)),
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
            water_plan_repository.clone(),
        )),
        list_water_seasons_use_case: Arc::new(ListWaterSeasonsUseCase::new(
            water_plan_repository.clone(),
        )),
        list_water_plan_entries_use_case: Arc::new(ListWaterPlanEntriesUseCase::new(
            water_plan_repository.clone(),
        )),
        create_water_plan_entry_use_case: Arc::new(CreateWaterPlanEntryUseCase::new(
            water_plan_repository.clone(),
        )),
        update_water_plan_entry_use_case: Arc::new(UpdateWaterPlanEntryUseCase::new(
            water_plan_repository.clone(),
        )),
        delete_water_plan_entry_use_case: Arc::new(DeleteWaterPlanEntryUseCase::new(
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
        list_my_offers_use_case: Arc::new(ListMyOffersUseCase::new(alwa_repository.clone())),
        create_market_use_case: Arc::new(CreateMarketUseCase::new(alwa_repository.clone())),
        update_market_use_case: Arc::new(UpdateMarketUseCase::new(alwa_repository.clone())),
        delete_market_use_case: Arc::new(DeleteMarketUseCase::new(alwa_repository.clone())),
        list_stored_prices_use_case: Arc::new(ListStoredPricesUseCase::new(
            alwa_repository.clone(),
        )),
        create_price_use_case: Arc::new(CreatePriceUseCase::new(alwa_repository.clone())),
        update_price_use_case: Arc::new(UpdatePriceUseCase::new(alwa_repository.clone())),
        delete_price_use_case: Arc::new(DeletePriceUseCase::new(alwa_repository.clone())),
        list_all_listings_use_case: Arc::new(ListAllListingsUseCase::new(alwa_repository.clone())),
        moderate_listing_use_case: Arc::new(ModerateListingUseCase::new(alwa_repository.clone())),
        delete_listing_use_case: Arc::new(DeleteListingUseCase::new(alwa_repository)),
    };

    let doctor_service: Arc<dyn Doctor> = Arc::new(HttpDoctor::new(&config.doctor.url)?);

    let doctor = DoctorFeature {
        ask_doctor_use_case: Arc::new(AskDoctorUseCase::new(
            farm_briefs,
            farm_history,
            doctor_service,
        )),
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
        doctor,
        staff,
        brief,
    })
}
