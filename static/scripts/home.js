/** @format */

let currentShift = 1;
let users = [];
let machines = [];
let flagReasons = [];
let downtimeReasons = [];
let scrapTypes = [];

document.addEventListener("DOMContentLoaded", function () {
	initializeDashboard();
});

async function initializeDashboard() {
	try {
		await loadReferenceData();
		await loadDashboardData();
		setupEventListeners();
		startAutoRefresh();
	} catch (error) {
		console.error("Dashboard initialization failed:", error);
		showNotification("Failed to initialize dashboard", "error");
	}
}

async function loadReferenceData() {
	try {
		const [usersResponse, machinesResponse, flagReasonsResponse, downtimeReasonsResponse, scrapTypesResponse] = await Promise.all([
			fetch("/api/users").then(handleApiResponse),
			fetch("/api/machines").then(handleApiResponse),
			fetch("/api/lookups/flag-reasons").then(handleApiResponse),
			fetch("/api/lookups/downtime-reasons").then(handleApiResponse),
			fetch("/api/lookups/scrap-types").then(handleApiResponse),
		]);

		users = usersResponse;
		machines = machinesResponse;
		flagReasons = flagReasonsResponse;
		downtimeReasons = downtimeReasonsResponse;
		scrapTypes = scrapTypesResponse;
	} catch (error) {
		console.error("Reference data loading failed:", error);
		showNotification("Failed to load reference data", "error");
	}
}

async function loadDashboardData() {
	try {
		updateCurrentShift();
		await loadShiftOverview();
		await loadRecentJobs();
		await loadRecentRolls();
		await loadRecentDowntimes();
		await loadRecentScraps();
	} catch (error) {
		console.error("Dashboard data loading failed:", error);
		showNotification("Failed to load dashboard data", "error");
	}
}

function updateCurrentShift() {
	const now = new Date();
	const hours = now.getHours();
	currentShift = hours >= 7 && hours < 19 ? 1 : 2;

	const shiftElement = document.getElementById("current-shift");
	if (shiftElement) {
		shiftElement.textContent = currentShift === 1 ? "Day Shift" : "Night Shift";
		shiftElement.className =
			currentShift === 1
				? "bg-blue-100 text-blue-800 px-3 py-1 rounded-full text-sm font-medium"
				: "bg-purple-100 text-purple-800 px-3 py-1 rounded-full text-sm font-medium";
	}
}

async function loadShiftOverview() {
	try {
		const today = new Date().toISOString().split("T")[0];

		const [jobsResponse, rollsResponse, downtimesResponse, scrapsResponse] = await Promise.all([
			fetch(`/api/jobs/filter-with-input-rolls?shift_id=${currentShift}&start_date=${today}&end_date=${today}`).then((r) => r.json()),
			fetch(`/api/output-rolls/filter?shift_id=${currentShift}&start_date=${today}&end_date=${today}`).then((r) => r.json()),
			fetch(`/api/downtimes/filter?shift_id=${currentShift}&start_date=${today}&end_date=${today}`).then((r) => r.json()),
			fetch(`/api/scraps/filter?shift_id=${currentShift}&start_date=${today}&end_date=${today}`).then((r) => r.json()),
		]);

		updateShiftStats(jobsResponse, rollsResponse, downtimesResponse, scrapsResponse);
	} catch (error) {
		console.error("Shift overview loading failed:", error);
		showNotification("Failed to load shift overview", "error");
	}
}

function updateShiftStats(jobsData, rollsData, downtimesData, scrapsData) {
	try {
		document.getElementById("shift-jobs").textContent = jobsData.total_count || 0;
		document.getElementById("shift-rolls").textContent = rollsData.total_count || 0;

		const totalDowntime = downtimesData.data ? downtimesData.data.reduce((sum, downtime) => sum + downtime.duration_minutes, 0) : 0;
		document.getElementById("shift-downtime").textContent = `${formatDowntime(totalDowntime)}`;

		const totalScrap = scrapsData.data ? scrapsData.data.reduce((sum, scrap) => sum + scrap.weight_kg, 0) : 0;
		document.getElementById("shift-scrap").textContent = `${formatWeight(totalScrap)}`;
	} catch (error) {
		console.error("Shift stats update failed:", error);
	}
}

async function loadRecentJobs() {
	try {
		const response = await fetch(`/api/jobs/filter-with-input-rolls?per_page=5&page=1`);
		const result = await response.json();
		renderRecentJobs(result.data);
	} catch (error) {
		console.error("Recent jobs loading failed:", error);
		document.getElementById("recent-jobs").innerHTML = '<div class="text-center text-red-500 py-4">Failed to load jobs</div>';
	}
}

function renderRecentJobs(jobs) {
	const container = document.getElementById("recent-jobs");

	if (!jobs || jobs.length === 0) {
		container.innerHTML = `
            <div class="text-center py-6">
                <i class="fas fa-inbox text-gray-400 text-2xl mb-2"></i>
                <p class="text-gray-500 text-sm">No jobs found</p>
            </div>
        `;
		return;
	}

	container.innerHTML = jobs
		.map((job) => {
			const machine = machines.find((m) => m.id === job.machine_id);
			const isActive = !job.completed_at;

			return `
            <div class="group flex items-center justify-between p-4 bg-white rounded-xl border border-gray-200 hover:border-blue-300 hover:shadow-md transition-all duration-200">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="flex-shrink-0 w-8 h-8 bg-blue-50 rounded-lg flex items-center justify-center">
                            <i class="fas fa-briefcase text-blue-600 text-sm"></i>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h4 class="text-sm font-semibold text-gray-900 truncate">${escapeHtml(job.production_order)}</h4>
                            <p class="text-xs text-gray-500 truncate">${escapeHtml(job.batch_roll_no)}</p>
                        </div>
                    </div>
                    <div class="flex items-center gap-3 text-xs text-gray-600">
                        <span class="inline-flex items-center gap-1">
                            <i class="fas fa-cogs"></i>
                            ${machine?.label || "Unknown"}
                        </span>
                        <span class="inline-flex items-center gap-1">
                            <i class="fas fa-clock"></i>
                            ${job.shift_id === 1 ? "Day" : "Night"}
                        </span>
                    </div>
                </div>
                <div class="flex items-center gap-4 ml-4">
                    <div class="text-right">
                        <div class="flex items-center gap-2 mb-1">
                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
								isActive ? "bg-green-100 text-green-800" : "bg-gray-100 text-gray-800"
							}">
                                <i class="fas fa-circle text-[6px] mr-1"></i>
                                ${isActive ? "Active" : "Completed"}
                            </span>
                            <span class="text-sm font-semibold text-gray-900">${job.total_rolls || 0}</span>
                        </div>
                        <div class="text-xs text-gray-500">${formatTime(job.created_at)}</div>
                    </div>
                    <div class="opacity-0 group-hover:opacity-100 transition-opacity">
                        <i class="fas fa-chevron-right text-gray-400"></i>
                    </div>
                </div>
            </div>
        `;
		})
		.join("");
}

async function loadRecentRolls() {
	try {
		const response = await fetch(`/api/output-rolls/filter?per_page=5&page=1`);
		const result = await response.json();
		renderRecentRolls(result.data);
	} catch (error) {
		console.error("Recent rolls loading failed:", error);
		document.getElementById("recent-rolls").innerHTML = '<div class="text-center text-red-500 py-4">Failed to load rolls</div>';
	}
}

function renderRecentRolls(rolls) {
	const container = document.getElementById("recent-rolls");

	if (!rolls || rolls.length === 0) {
		container.innerHTML = `
            <div class="text-center py-6">
                <i class="fas fa-layer-group text-gray-400 text-2xl mb-2"></i>
                <p class="text-gray-500 text-sm">No rolls found</p>
            </div>
        `;
		return;
	}

	container.innerHTML = rolls
		.map((roll) => {
			const flagReason = roll.flag_reason_id ? flagReasons.find((fr) => fr.id === roll.flag_reason_id) : null;
			const hasFlags = roll.number_of_flags > 0;

			return `
            <div class="group flex items-center justify-between p-4 bg-white rounded-xl border border-gray-200 hover:border-green-300 hover:shadow-md transition-all duration-200">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="flex-shrink-0 w-8 h-8 bg-green-50 rounded-lg flex items-center justify-center">
                            <i class="fas fa-layer-group text-green-600 text-sm"></i>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h4 class="text-sm font-semibold text-gray-900 truncate">${escapeHtml(roll.output_roll_no)}</h4>
                            <div class="flex items-center gap-2 mt-1">
                                <span class="text-xs text-gray-600">Weight: <strong>${formatWeight(roll.final_weight)}</strong></span>
                                <span class="text-xs text-gray-600">Meters: <strong>${roll.final_meter}</strong></span>
                                ${
									hasFlags
										? `
                                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                                        <i class="fas fa-flag text-[8px] mr-1"></i>
                                        ${roll.number_of_flags} flags
                                    </span>
                                `
										: ""
								}
                            </div>
                        </div>
                    </div>
                    ${
						flagReason
							? `
                        <div class="flex items-center gap-1 text-xs text-gray-600">
                            <i class="fas fa-info-circle text-orange-500"></i>
                            <span class="truncate">${escapeHtml(flagReason.name)}</span>
                        </div>
                    `
							: ""
					}
                </div>
                <div class="flex items-center gap-4 ml-4">
                    <div class="text-right">
                        <div class="text-sm font-medium text-gray-900 mb-1">${roll.shift_id === 1 ? "Day" : "Night"}</div>
                        <div class="text-xs text-gray-500">${formatTime(roll.created_at)}</div>
                    </div>
                    <div class="opacity-0 group-hover:opacity-100 transition-opacity">
                        <i class="fas fa-chevron-right text-gray-400"></i>
                    </div>
                </div>
            </div>
        `;
		})
		.join("");
}

async function loadRecentDowntimes() {
	try {
		const response = await fetch(`/api/downtimes/filter?per_page=5&page=1`);
		const result = await response.json();
		renderRecentDowntimes(result.data);
	} catch (error) {
		console.error("Recent downtimes loading failed:", error);
		document.getElementById("recent-downtimes").innerHTML = '<div class="text-center text-red-500 py-4">Failed to load downtimes</div>';
	}
}

function renderRecentDowntimes(downtimes) {
	const container = document.getElementById("recent-downtimes");

	if (!downtimes || downtimes.length === 0) {
		container.innerHTML = `
            <div class="text-center py-6">
                <i class="fas fa-clock text-gray-400 text-2xl mb-2"></i>
                <p class="text-gray-500 text-sm">No downtimes found</p>
            </div>
        `;
		return;
	}

	container.innerHTML = downtimes
		.map((downtime) => {
			const reason = downtimeReasons.find((dr) => dr.id === downtime.downtime_reason_id);

			return `
            <div class="group flex items-center justify-between p-4 bg-white rounded-xl border border-gray-200 hover:border-orange-300 hover:shadow-md transition-all duration-200">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="flex-shrink-0 w-8 h-8 bg-orange-50 rounded-lg flex items-center justify-center">
                            <i class="fas fa-clock text-orange-600 text-sm"></i>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h4 class="text-sm font-semibold text-gray-900">${formatDowntime(downtime.duration_minutes)}</h4>
                            <p class="text-xs text-gray-600 mt-1">
                                ${formatTime(downtime.start_time)} - ${formatTime(downtime.end_time)}
                            </p>
                        </div>
                    </div>
                    ${
						reason
							? `
                        <div class="flex items-center gap-1 text-xs text-gray-600">
                            <i class="fas fa-comment text-blue-500"></i>
                            <span class="truncate">${escapeHtml(reason.name)}</span>
                        </div>
                    `
							: ""
					}
                </div>
                <div class="flex items-center gap-4 ml-4">
                    <div class="text-right">
                        <div class="text-sm font-medium text-gray-900 mb-1">${downtime.shift_id === 1 ? "Day" : "Night"}</div>
                        <div class="text-xs text-gray-500">${formatTime(downtime.created_at)}</div>
                    </div>
                    <div class="opacity-0 group-hover:opacity-100 transition-opacity">
                        <i class="fas fa-chevron-right text-gray-400"></i>
                    </div>
                </div>
            </div>
        `;
		})
		.join("");
}

async function loadRecentScraps() {
	try {
		const response = await fetch(`/api/scraps/filter?per_page=5&page=1`);
		const result = await response.json();
		renderRecentScraps(result.data);
	} catch (error) {
		console.error("Recent scraps loading failed:", error);
		document.getElementById("recent-scraps").innerHTML = '<div class="text-center text-red-500 py-4">Failed to load scraps</div>';
	}
}

function renderRecentScraps(scraps) {
	const container = document.getElementById("recent-scraps");

	if (!scraps || scraps.length === 0) {
		container.innerHTML = `
            <div class="text-center py-6">
                <i class="fas fa-trash text-gray-400 text-2xl mb-2"></i>
                <p class="text-gray-500 text-sm">No scraps found</p>
            </div>
        `;
		return;
	}

	container.innerHTML = scraps
		.map((scrap) => {
			const scrapType = scrapTypes.find((st) => st.id === scrap.scrap_type_id);

			return `
            <div class="group flex items-center justify-between p-4 bg-white rounded-xl border border-gray-200 hover:border-red-300 hover:shadow-md transition-all duration-200">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="flex-shrink-0 w-8 h-8 bg-red-50 rounded-lg flex items-center justify-center">
                            <i class="fas fa-trash text-red-600 text-sm"></i>
                        </div>
                        <div class="min-w-0 flex-1">
                            <h4 class="text-sm font-semibold text-gray-900">${formatWeight(scrap.weight_kg)}</h4>
                            <p class="text-xs text-gray-600 mt-1">
                                ${scrapType ? escapeHtml(scrapType.name) : "Unknown Type"}
                            </p>
                        </div>
                    </div>
                    <div class="flex items-center gap-1 text-xs text-gray-600">
                        <i class="fas fa-calendar"></i>
                        <span>${formatTime(scrap.time)}</span>
                    </div>
                </div>
                <div class="flex items-center gap-4 ml-4">
                    <div class="text-right">
                        <div class="text-sm font-medium text-gray-900 mb-1">${scrap.shift_id === 1 ? "Day" : "Night"}</div>
                        <div class="text-xs text-gray-500">${formatTime(scrap.created_at)}</div>
                    </div>
                    <div class="opacity-0 group-hover:opacity-100 transition-opacity">
                        <i class="fas fa-chevron-right text-gray-400"></i>
                    </div>
                </div>
            </div>
        `;
		})
		.join("");
}

async function searchRoll() {
	const rollNumber = document.getElementById("roll-lookup-input").value.trim();

	if (!rollNumber) {
		showNotification("Please enter a roll number", "warning");
		return;
	}

	const searchBtn = document.getElementById("search-roll-btn");
	setButtonLoading(searchBtn, true);

	try {
		const response = await fetch(`/api/output-rolls/filter?output_roll_no=${encodeURIComponent(rollNumber)}&per_page=1`);
		const result = await response.json();

		if (result.data.length === 0) {
			showNotification("Roll not found", "warning");
			return;
		}

		const roll = result.data[0];
		showRollDetails(roll);
		closeLookupModal();
	} catch (error) {
		console.error("Roll search failed:", error);
		showNotification("Failed to search roll", "error");
	} finally {
		setButtonLoading(searchBtn, false);
	}
}

function showRollDetails(roll) {
	const createdBy = users.find((u) => u.id === roll.created_by);
	const flagReason = roll.flag_reason_id ? flagReasons.find((fr) => fr.id === roll.flag_reason_id) : null;
	const isCompleted = roll.final_weight > 0;

	document.getElementById("detail-roll-no").textContent = roll.output_roll_no;
	document.getElementById("detail-status").innerHTML = `
        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
			isCompleted ? "bg-green-100 text-green-800" : "bg-yellow-100 text-yellow-800"
		}">
            <i class="fas fa-circle text-[6px] mr-1"></i>
            ${isCompleted ? "Completed" : "Pending"}
        </span>
    `;
	document.getElementById("detail-weight").textContent = formatWeight(roll.final_weight);
	document.getElementById("detail-meters").textContent = `${roll.final_meter} m`;
	document.getElementById("detail-job-id").textContent = roll.job_id;
	document.getElementById("detail-shift").textContent = roll.shift_id === 1 ? "Day" : "Night";
	document.getElementById("detail-created-by").textContent = createdBy?.full_name || "System";
	document.getElementById("detail-created-time").textContent = formatTime(roll.created_at);
	document.getElementById("detail-flag-reason").innerHTML = flagReason
		? `
        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-orange-100 text-orange-800">
            <i class="fas fa-flag text-[8px] mr-1"></i>
            ${escapeHtml(flagReason.name)}
        </span>
    `
		: '<span class="text-gray-500">None</span>';
	document.getElementById("detail-sap-status").innerHTML = `
        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            <i class="fas fa-sync-alt text-[8px] mr-1"></i>
            Not Integrated
        </span>
    `;

	document.getElementById("roll-details-modal").style.display = "flex";
}

function openLookupModal() {
	document.getElementById("roll-lookup-modal").style.display = "flex";
	document.getElementById("roll-lookup-input").value = "";
	document.getElementById("roll-lookup-input").focus();
}

function closeLookupModal() {
	document.getElementById("roll-lookup-modal").style.display = "none";
}

function closeRollDetailsModal() {
	document.getElementById("roll-details-modal").style.display = "none";
}

function setupEventListeners() {
	document.getElementById("open-roll-lookup").addEventListener("click", openLookupModal);
	document.getElementById("close-lookup-modal").addEventListener("click", closeLookupModal);
	document.getElementById("close-roll-modal").addEventListener("click", closeRollDetailsModal);
	document.getElementById("close-roll-details").addEventListener("click", closeRollDetailsModal);
	document.getElementById("search-roll-btn").addEventListener("click", searchRoll);

	document.getElementById("roll-lookup-input").addEventListener("keypress", function (e) {
		if (e.key === "Enter") {
			searchRoll();
		}
	});
}

function startAutoRefresh() {
	setInterval(() => {
		loadDashboardData();
	}, 30000);
}
