use crate::{
    backend::{handlers::*, middlewares::*, templates::*},
    sap::get_batch_availability_handler,
};
use actix_web::web;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn init_routes(
    cfg: &mut web::ServiceConfig,
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
) {
    // User routes
    cfg.service(
        web::scope("/api/users")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_users)),
            )
            .service(
                web::resource("/me")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(get_me)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_user)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_user)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_user)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_users)),
            ),
    );

    // Permission routes
    cfg.service(
        web::scope("/api/permissions")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_permissions)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_permission)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_permission)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_permission)),
            ),
    );

    // Role routes
    cfg.service(
        web::scope("/api/roles")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_roles)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "roles",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_role)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "roles",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_role)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "roles",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_role)),
            ),
    );

    // Auth routes
    cfg.service(
        web::scope("/auth")
            .route("/signin", web::post().to(signin_user))
            .route("/signin", web::get().to(signin_page))
            .route("/signout", web::get().to(signout_user)),
    );

    // Job routes
    cfg.service(
        web::scope("/api/jobs")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "jobs",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_jobs)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "jobs",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_job)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "jobs",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_job)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "jobs",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_job)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "jobs",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_jobs_with_pending_input_rolls)),
            ),
    );

    // Input Roll routes
    cfg.service(
        web::scope("/api/input-rolls")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_input_rolls)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_input_roll)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_input_roll)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_input_roll)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_input_rolls)),
            )
            .service(
                web::resource("/filter-with-stats")
                    .wrap(CheckRead {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_input_rolls_with_stats)),
            )
            .service(
                web::resource("/details")
                    .wrap(CheckRead {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(input_roll_details)),
            )
            .service(
                web::resource("/end")
                    .wrap(CheckUpdate {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(end_input_roll)),
            ),
    );

    // Output Roll routes
    cfg.service(
        web::scope("/api/output-rolls")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_output_rolls)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_output_roll)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_output_roll)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_output_roll)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_output_rolls)),
            )
            .service(
                web::resource("/details")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(output_roll_details)),
            ),
    );

    // Downtime routes
    cfg.service(
        web::scope("/api/downtimes")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "downtimes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_downtimes)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "downtimes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_downtime)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "downtimes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_downtime)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "downtimes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_downtime)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "downtimes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_downtimes)),
            ),
    );

    // Scrap routes
    cfg.service(
        web::scope("/api/scraps")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "scraps",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_scraps)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "scraps",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_scrap)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "scraps",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_scrap)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "scraps",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_scrap)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "scraps",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_scraps)),
            ),
    );

    // Ink Usage routes
    cfg.service(
        web::scope("/api/ink-usages")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "ink_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_ink_usages)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "ink_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_ink_usage)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "ink_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_ink_usage)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "ink_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_ink_usage)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "ink_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_ink_usages)),
            ),
    );

    // Process Order routes
    cfg.service(
        web::scope("/api/process_order")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(get_process_orders)),
            )
            .service(
                web::resource("/all")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(get_all_process_orders)),
            ),
    );

    // Solvent Usage routes
    cfg.service(
        web::scope("/api/solvent-usages")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "solvent_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_solvent_usages)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "solvent_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_solvent_usage)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "solvent_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_solvent_usage)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "solvent_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_solvent_usage)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "solvent_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_solvent_usages)),
            ),
    );

    // Lookup routes
    cfg.service(
        web::scope("/api/lookups")
            .service(
                web::resource("/shifts")
                    .wrap(CheckRead {
                        model: "shifts",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_shifts)),
            )
            .service(
                web::resource("/shifts/create")
                    .wrap(CheckCreate {
                        model: "shifts",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_shift)),
            )
            .service(
                web::resource("/shifts/update")
                    .wrap(CheckUpdate {
                        model: "shifts",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_shift)),
            )
            .service(
                web::resource("/shifts/delete")
                    .wrap(CheckDelete {
                        model: "shifts",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_shift)),
            )
            .service(
                web::resource("/colours")
                    .wrap(CheckRead {
                        model: "colours",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_colours)),
            )
            .service(
                web::resource("/colours/create")
                    .wrap(CheckCreate {
                        model: "colours",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_colour)),
            )
            .service(
                web::resource("/colours/update")
                    .wrap(CheckUpdate {
                        model: "colours",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_colour)),
            )
            .service(
                web::resource("/colours/delete")
                    .wrap(CheckDelete {
                        model: "colours",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_colour)),
            )
            .service(
                web::resource("/solvent-types")
                    .wrap(CheckRead {
                        model: "solvent_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_solvent_types)),
            )
            .service(
                web::resource("/solvent-types/create")
                    .wrap(CheckCreate {
                        model: "solvent_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_solvent_type)),
            )
            .service(
                web::resource("/solvent-types/update")
                    .wrap(CheckUpdate {
                        model: "solvent_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_solvent_type)),
            )
            .service(
                web::resource("/solvent-types/delete")
                    .wrap(CheckDelete {
                        model: "solvent_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_solvent_type)),
            )
            .service(
                web::resource("/scrap-types")
                    .wrap(CheckRead {
                        model: "scrap_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_scrap_types)),
            )
            .service(
                web::resource("/scrap-types/create")
                    .wrap(CheckCreate {
                        model: "scrap_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_scrap_type)),
            )
            .service(
                web::resource("/scrap-types/update")
                    .wrap(CheckUpdate {
                        model: "scrap_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_scrap_type)),
            )
            .service(
                web::resource("/scrap-types/delete")
                    .wrap(CheckDelete {
                        model: "scrap_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_scrap_type)),
            )
            .service(
                web::resource("/downtime-reasons")
                    .wrap(CheckRead {
                        model: "downtime_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_downtime_reasons)),
            )
            .service(
                web::resource("/downtime-reasons/create")
                    .wrap(CheckCreate {
                        model: "downtime_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_downtime_reason)),
            )
            .service(
                web::resource("/downtime-reasons/update")
                    .wrap(CheckUpdate {
                        model: "downtime_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_downtime_reason)),
            )
            .service(
                web::resource("/downtime-reasons/delete")
                    .wrap(CheckDelete {
                        model: "downtime_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_downtime_reason)),
            )
            .service(
                web::resource("/flag-reasons")
                    .wrap(CheckRead {
                        model: "flag_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_flag_reasons)),
            )
            .service(
                web::resource("/flag-reasons/section/{section_id}")
                    .wrap(CheckRead {
                        model: "flag_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(flag_reasons_by_section)),
            )
            .service(
                web::resource("/flag-reasons/create")
                    .wrap(CheckCreate {
                        model: "flag_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_flag_reason)),
            )
            .service(
                web::resource("/flag-reasons/update")
                    .wrap(CheckUpdate {
                        model: "flag_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_flag_reason)),
            )
            .service(
                web::resource("/flag-reasons/delete")
                    .wrap(CheckDelete {
                        model: "flag_reasons",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_flag_reason)),
            )
            .service(
                web::resource("/po-code-sections")
                    .wrap(CheckRead {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(get_po_code_sections)),
            )
            .service(
                web::resource("/po-code-sections/create")
                    .wrap(CheckCreate {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(add_po_code_section)),
            )
            .service(
                web::resource("/po-code-sections/delete")
                    .wrap(CheckDelete {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(remove_po_code_section)),
            )
            .service(
                web::resource("/po-codes")
                    .wrap(CheckRead {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_po_codes)),
            )
            .service(
                web::resource("/po-codes/create")
                    .wrap(CheckCreate {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_po_code)),
            )
            .service(
                web::resource("/po-codes/update")
                    .wrap(CheckUpdate {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_po_code)),
            )
            .service(
                web::resource("/po-codes/delete")
                    .wrap(CheckDelete {
                        model: "po_codes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_po_code)),
            ),
    );

    // Section routes
    cfg.service(
        web::scope("/api/sections")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_sections)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_section)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_section)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_section)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_sections)),
            )
            .service(
                web::resource("/update-po-codes")
                    .wrap(CheckUpdate {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_po_code_sections)),
            ),
    );

    // Machine routes
    cfg.service(
        web::scope("/api/machines")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "machines",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_machines)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "machines",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_machine)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "machines",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_machine)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "machines",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_machine)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "machines",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_machines)),
            ),
    );

    // Material routes
    cfg.service(
        web::scope("/api/materials")
            .service(
                web::resource("")
                    .wrap(CheckRead {
                        model: "materials",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(all_materials)),
            )
            .service(
                web::resource("/create")
                    .wrap(CheckCreate {
                        model: "materials",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::post().to(create_material)),
            )
            .service(
                web::resource("/update")
                    .wrap(CheckUpdate {
                        model: "materials",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::put().to(update_material)),
            )
            .service(
                web::resource("/delete")
                    .wrap(CheckDelete {
                        model: "materials",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::delete().to(delete_material)),
            )
            .service(
                web::resource("/filter")
                    .wrap(CheckRead {
                        model: "materials",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(filter_materials)),
            ),
    );

    // Material Availability routes
    cfg.service(
        web::scope("/api/materials-availability").service(
            web::resource("")
                .wrap(CheckRead {
                    model: "materials",
                    conn_data: conn_data.clone(),
                })
                .route(web::get().to(get_batch_availability_handler)),
        ),
    );

    // Others routes
    cfg.service(
        web::scope("")
            .service(
                web::resource("/")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(home_page)),
            )
            .service(web::resource("/upload/{name}").route(web::post().to(upload_app)))
            .service(web::resource("/download/{name}").route(web::get().to(download_app)))
            .service(
                web::resource("/users")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(user_page)),
            )
            .service(
                web::resource("/jobs")
                    .wrap(CheckRead {
                        model: "jobs",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(jobs_page)),
            )
            .service(
                web::resource("/output-rolls")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(output_rolls_page)),
            )
            .service(
                web::resource("/input-rolls")
                    .wrap(CheckRead {
                        model: "input_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(input_rolls_page)),
            )
            .service(
                web::resource("/roles")
                    .wrap(CheckRead {
                        model: "roles",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(roles_page)),
            )
            .service(
                web::resource("/scrap")
                    .wrap(CheckRead {
                        model: "scraps",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(scrap_page)),
            )
            .service(
                web::resource("/downtime")
                    .wrap(CheckRead {
                        model: "downtimes",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(downtime_page)),
            )
            .service(
                web::resource("/production")
                    .wrap(CheckRead {
                        model: "output_rolls",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(production_page)),
            )
            .service(
                web::resource("/machines")
                    .wrap(CheckRead {
                        model: "machines",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(machine_page)),
            )
            .service(
                web::resource("/materials")
                    .wrap(CheckRead {
                        model: "materials",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(material_page)),
            )
            .service(
                web::resource("/sections")
                    .wrap(CheckRead {
                        model: "sections",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(section_page)),
            )
            .service(
                web::resource("/lookups")
                    .wrap(CheckRead {
                        model: "solvent_types",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(lookup_page)),
            )
            .service(
                web::resource("/consumables")
                    .wrap(CheckRead {
                        model: "ink_usages",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(consumable_page)),
            )
            .service(
                web::resource("/settings")
                    .wrap(CheckRead {
                        model: "users",
                        conn_data: conn_data.clone(),
                    })
                    .route(web::get().to(settings_page)),
            )
            .route("/whois", web::get().to(whois_data))
            .route("/logout", web::get().to(logout))
            .route("/{_:.*}", web::get().to(signin_page)),
    );
}
