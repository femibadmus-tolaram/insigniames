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

		users = usersResponse;
		machines = machinesResponse;

		populateSelect("filter-user", users, "full_name", "All Users");
		populateSelect("filter-machine", machines, "name", "All Machines");
		populateSelect("machine", machines, "name", "Select Machine");
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
			'<tr><td colspan="11" class="text-center text-red-500 py-4">Failed to load job records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "jobs-table");
	}
}

function updateJobStats(jobs) {
	const totalRecords = jobs.length;
	const activeJobs = jobs.filter((job) => !job.completed_at).length;

	const totalRolls = jobs.reduce((sum, job) => sum + (job.total_rolls || 0), 0);
	const pendingRolls = jobs.reduce((sum, job) => sum + (job.pending_rolls || 0), 0);
	const totalWeight = jobs.reduce((sum, job) => sum + (job.total_weight || 0), 0);

	document.getElementById("total-jobs").textContent = totalRecords;
	document.getElementById("active-jobs").textContent = activeJobs;
	document.getElementById("total-rolls").textContent = totalRolls;
	document.getElementById("pending-rolls").textContent = pendingRolls;
	document.getElementById("total-weight").textContent = formatWeight(totalWeight);
}

function renderJobs(jobsToRender) {
	const tbody = document.getElementById("jobs-table-body");
	if (!jobsToRender || jobsToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="11" class="text-center text-gray-500 py-4">No job records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	jobsToRender.forEach((job) => {
		const createdBy = users.find((u) => u.id === job.created_by);
		const machine = machines.find((m) => m.id === job.machine_id);

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
			<td class="py-3 px-4">
				<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
					job.completed_at ? "bg-gray-100 text-gray-800" : "bg-green-100 text-green-800"
				}">
					${job.completed_at ? "Completed" : "Active"}
				</span>
			</td>
			<td class="py-3 px-4">${escapeHtml(job.production_order)}</td>
			<td class="py-3 px-4">${escapeHtml(job.batch_roll_no)}</td>
			<td class="py-3 px-4">${job.shift_id === 1 ? "Day" : "Night"}</td>
			<td class="py-3 px-4">${escapeHtml(machine?.name || "Unknown")}</td>
			<td class="py-3 px-4 text-center">
				<span class="inline-flex items-center gap-1">
					<i class="fas fa-layer-group text-blue-600"></i>
					${job.total_rolls || 0}
				</span>
			</td>
			<td class="py-3 px-4 text-center">
				<span class="inline-flex items-center gap-1 ${job.pending_rolls > 0 ? "text-yellow-600" : "text-gray-600"}">
					<i class="fas fa-clock"></i>
					${job.pending_rolls || 0}
				</span>
			</td>
			<td class="py-3 px-4 text-center">
				<span class="inline-flex items-center gap-1 text-green-600">
					<i class="fas fa-weight-hanging"></i>
					${formatWeight(job.total_weight || 0)}
				</span>
			</td>
			<td class="py-3 px-4">${formatDateTime(job.last_updated || job.updated_at)}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2">
					<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${job.id}">
						<i class="fas fa-edit"></i>
					</button>
					<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${job.id}">
						<i class="fas fa-trash"></i>
					</button>
				</div>
			</td>
		`;

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => editJob(btn.dataset.id));
	});
	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			deleteJob(btn.dataset.id);
		});
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
		const batchRollNo = document.getElementById("filter-batch-roll-no").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const machineFilter = document.getElementById("filter-machine").value;
		const userFilter = document.getElementById("filter-user").value;
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;

		if (productionOrder) params.append("production_order", productionOrder);
		if (batchRollNo) params.append("batch_roll_no", batchRollNo);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (machineFilter) params.append("machine_id", machineFilter);
		if (userFilter) params.append("created_by", userFilter);
		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
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
	document.getElementById("filter-batch-roll-no").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-machine").value = "";
	document.getElementById("filter-user").value = "";
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";

	currentPage = 1;
	applyFilters();
}

function openModal(jobId = null) {
	const modal = document.getElementById("job-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("job-form");

	if (jobId) {
		title.textContent = "Update Job";
		const job = jobs.find((j) => j.id === parseInt(jobId));
		if (job) {
			populateForm(job);
		}
	} else {
		title.textContent = "Add Job";
		form.reset();
		document.getElementById("job-id").value = "";
	}

	modal.style.display = "flex";
}

function closeModal() {
	document.getElementById("job-modal").style.display = "none";
}

function populateForm(job) {
	document.getElementById("job-id").value = job.id;
	document.getElementById("production-order").value = job.production_order;
	document.getElementById("batch-roll-no").value = job.batch_roll_no;
	document.getElementById("start-weight").value = job.start_weight;
	document.getElementById("start-meter").value = job.start_meter;
	document.getElementById("shift").value = job.shift_id;
	document.getElementById("machine").value = job.machine_id;
}

function handleFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const jobId = document.getElementById("job-id").value;
	const formData = {
		production_order: document.getElementById("production-order").value,
		batch_roll_no: document.getElementById("batch-roll-no").value,
		start_weight: parseFloat(document.getElementById("start-weight").value),
		start_meter: parseFloat(document.getElementById("start-meter").value),
		shift_id: parseInt(document.getElementById("shift").value),
		machine_id: parseInt(document.getElementById("machine").value),
	};

	if (jobId) {
		formData.id = parseInt(jobId);
		updateJob(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function updateJob(jobData) {
	try {
		const response = await fetch("/api/jobs/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(jobData),
		});
		const updatedJob = await handleApiResponse(response);

		// Update local data
		const jobIndex = jobs.findIndex((j) => j.id === jobData.id);
		if (jobIndex !== -1) {
			jobs[jobIndex] = { ...jobs[jobIndex], ...updatedJob };
			updateJobStats(jobs);
			renderJobs(jobs);
		}

		showNotification("Job updated successfully!", "success");
		closeModal();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editJob(jobId) {
	openModal(jobId);
}

async function deleteJob(jobId) {
	if (!confirm("Are you sure you want to delete this job?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${jobId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/jobs/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(jobId) }),
		});
		await handleApiResponse(response);

		// Remove from local data
		jobs = jobs.filter((j) => j.id !== parseInt(jobId));
		totalCount--;
		updateJobStats(jobs);
		renderJobs(jobs);
		renderPagination();

		showNotification("Job deleted successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

async function exportToExcel() {
	const exportBtn = document.getElementById("export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const params = new URLSearchParams();
		const productionOrder = document.getElementById("filter-production-order").value;
		const batchRollNo = document.getElementById("filter-batch-roll-no").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const machineFilter = document.getElementById("filter-machine").value;
		const userFilter = document.getElementById("filter-user").value;
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;

		if (productionOrder) params.append("production_order", productionOrder);
		if (batchRollNo) params.append("batch_roll_no", batchRollNo);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (machineFilter) params.append("machine_id", machineFilter);
		if (userFilter) params.append("created_by", userFilter);
		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);

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
				"Job ID": job.id,
				Status: job.completed_at ? "Completed" : "Active",
				"Production Order": job.production_order,
				"Batch Roll No": job.batch_roll_no,
				Shift: job.shift_id === 1 ? "Day" : "Night",
				Machine: machine?.name || "Unknown",
				"Total Rolls": job.total_rolls || 0,
				"Pending Rolls": job.pending_rolls || 0,
				"Total Weight": formatWeight(job.total_weight || 0),
				"Start Weight (kg)": job.start_weight,
				"Start Meter": job.start_meter,
				"Created By": createdBy?.full_name || "System",
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
	document.getElementById("close-modal").addEventListener("click", closeModal);
	document.getElementById("cancel-btn").addEventListener("click", closeModal);
	document.getElementById("job-form").addEventListener("submit", handleFormSubmit);

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
