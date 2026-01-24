/** @format */

let jobs = [];
let users = [];
let machines = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;
document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadFilterOptions();
	await loadJobs();
	setupEventListeners();
}

async function loadFilterOptions() {
	try {
		const [usersResponse, machinesResponse] = await Promise.all([
			fetch("/api/users").then(handleApiResponse),
			fetch("/api/machines").then(handleApiResponse),
		]);

		users = Array.isArray(usersResponse) ? usersResponse : usersResponse.data || [];
		machines = Array.isArray(machinesResponse) ? machinesResponse : machinesResponse.data || [];

		populateSelect("filter-user", users, "full_name", "All Users");
		populateSelect("filter-machine", machines, "name", "All Machines");
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadJobs() {
	showLoading(true, "jobs-table");
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/jobs/filter?${params}`);
		const result = await handleApiResponse(response);

		jobs = result.data;
		totalCount = result.total_count;

		updateJobStats(jobs);
		renderJobs(jobs);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		document.getElementById("jobs-table-body").innerHTML =
			'<tr><td colspan="10" class="text-center text-red-500 py-4">Failed to load job records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "jobs-table");
	}
}

function updateJobStats(jobs) {
	const totalRecords = jobs.length;
	const uniqueMachines = new Set(jobs.map((job) => job.machine_id)).size;
	const uniqueOrders = new Set(jobs.map((job) => job.production_order)).size;

	document.getElementById("total-entries").textContent = totalRecords;
	document.getElementById("unique-machines").textContent = uniqueMachines;
	document.getElementById("unique-orders").textContent = uniqueOrders;
}

function renderJobs(jobsToRender) {
	const tbody = document.getElementById("jobs-table-body");
	if (!jobsToRender || jobsToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="10" class="text-center text-gray-500 py-4">No job records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	jobsToRender.forEach((job) => {
		const createdBy = users.find((u) => u.id === job.created_by);
		const machine = machines.find((m) => m.id === job.machine_id);

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
			<td class="py-3 px-4">${escapeHtml(job.production_order)}</td>
			<td class="py-3 px-4">${escapeHtml(job.batch || "-")}</td>
			<td class="py-3 px-4">${escapeHtml(machine?.name || "Unknown")}</td>
			<td class="py-3 px-4">${job.shift_id === 1 ? "Day" : "Night"}</td>
			<td class="py-3 px-4">${job.start_datetime ? formatDateTime(job.start_datetime) : "-"}</td>
			<td class="py-3 px-4">${escapeHtml(job.start_weight || "-")}</td>
			<td class="py-3 px-4">${typeof job.start_meter === "number" ? formatMeter(job.start_meter) : "-"}</td>
			<td class="py-3 px-4">${escapeHtml(job.material_number || "-")}</td>
			<td class="py-3 px-4">${formatDateTime(job.last_updated || job.updated_at)}</td>
			<td class="py-3 px-4">${escapeHtml(createdBy?.full_name || "System")}</td>
		`;

		tbody.appendChild(row);
	});
}

function renderPagination() {
	const totalPages = Math.ceil(totalCount / itemsPerPage);
	const paginationContainer = document.getElementById("pagination");

	if (totalPages <= 1) {
		paginationContainer.innerHTML = "";
		return;
	}

	const startItem = (currentPage - 1) * itemsPerPage + 1;
	const endItem = Math.min(currentPage * itemsPerPage, totalCount);

	let paginationHTML = `
		<div class="flex items-center gap-4">
			<div class="text-sm text-gray-600">
				Showing ${startItem} to ${endItem} of ${totalCount} entries
			</div>
			<div class="flex gap-1">
				<button class="pagination-btn" ${currentPage === 1 ? "disabled" : ""} id="prev-page">
					<i class="fas fa-chevron-left"></i>
				</button>
				<button class="pagination-btn" ${currentPage === totalPages ? "disabled" : ""} id="next-page">
					<i class="fas fa-chevron-right"></i>
				</button>
			</div>
		</div>
	`;

	paginationContainer.innerHTML = paginationHTML;

	document.getElementById("prev-page")?.addEventListener("click", () => {
		if (currentPage > 1) {
			currentPage--;
			applyFilters();
		}
	});

	document.getElementById("next-page")?.addEventListener("click", () => {
		if (currentPage < totalPages) {
			currentPage++;
			applyFilters();
		}
	});
}

async function applyFilters() {
	const applyBtn = document.getElementById("apply-filter");
	setButtonLoading(applyBtn, true);
	showLoading(true, "jobs-table");

	try {
		const params = new URLSearchParams();
		const productionOrder = document.getElementById("filter-production-order").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const machineFilter = document.getElementById("filter-machine").value;
		const userFilter = document.getElementById("filter-user").value;

		if (productionOrder) params.append("production_order", productionOrder);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (machineFilter) params.append("machine_id", machineFilter);
		if (userFilter) params.append("created_by", userFilter);
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/jobs/filter?${params}`);
		const result = await handleApiResponse(response);

		jobs = result.data;
		totalCount = result.total_count;

		updateJobStats(jobs);
		renderJobs(jobs);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		showLoading(false, "jobs-table");
	}
}

function clearFilters() {
	document.getElementById("filter-production-order").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-machine").value = "";
	document.getElementById("filter-user").value = "";

	currentPage = 1;
	applyFilters();
}

async function exportToExcel() {
	const exportBtn = document.getElementById("export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const params = new URLSearchParams();
		const productionOrder = document.getElementById("filter-production-order").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const machineFilter = document.getElementById("filter-machine").value;
		const userFilter = document.getElementById("filter-user").value;

		if (productionOrder) params.append("production_order", productionOrder);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (machineFilter) params.append("machine_id", machineFilter);
		if (userFilter) params.append("created_by", userFilter);
		const response = await fetch(`/api/jobs/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredJobs = result.data;

		if (filteredJobs.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredJobs.map((job) => {
			const createdBy = users.find((u) => u.id === job.created_by);
			const machine = machines.find((m) => m.id === job.machine_id);

			return {
								"Production Order": job.production_order,
				Batch: job.batch || "-",
				Machine: machine?.name || "Unknown",
				Shift: job.shift_id === 1 ? "Day" : "Night",
				"Start Weight": job.start_weight || "-",
				"Start Meter": typeof job.start_meter === "number" ? job.start_meter : "-",
				"Material Number": job.material_number || "-",
				"Created By": createdBy?.full_name || "System",
				"Start Time": job.start_datetime ? formatDateTime(job.start_datetime) : "-",
				"Created At": formatDateTime(job.created_at),
				"Last Updated": formatDateTime(job.last_updated || job.updated_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Job Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "job_records.xlsx");

		showNotification("Job records exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", applyFilters);
	document.getElementById("clear-filter").addEventListener("click", clearFilters);

	const exportBtn = document.getElementById("export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}

	document.getElementById("per-page").addEventListener("change", function () {
		itemsPerPage = parseInt(this.value);
		currentPage = 1;
		applyFilters();
	});
}











