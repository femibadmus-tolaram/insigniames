/** @format */
let downtimes = [];
let downtimeReasons = [];
let users = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;
let shifts = [
	{ id: 1, name: "Day" },
	{ id: 2, name: "Night" },
];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadFilterOptions();
	await loadDowntimes();
	setupEventListeners();
}

async function loadFilterOptions() {
	try {
		const [reasonsResponse, usersResponse] = await Promise.all([
			fetch("/api/lookups/downtime-reasons").then(handleApiResponse),
			fetch("/api/users").then(handleApiResponse),
		]);

		downtimeReasons = reasonsResponse;
		users = usersResponse;

		populateSelect("filter-reason", downtimeReasons, "name", "All Reasons");
		populateSelect("filter-user", users, "full_name", "All Users");
		populateSelect("downtime-reason", downtimeReasons, "name", "Select Reason");
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadDowntimes() {
	showLoading(true, "downtime-table");
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/downtimes/filter?${params}`);
		const result = await handleApiResponse(response);

		downtimes = result.data;
		totalCount = result.total_count;
		updateDowntimeStats(downtimes);
		renderDowntimes(downtimes);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		document.getElementById("downtime-table-body").innerHTML =
			'<tr><td colspan="8" class="text-center text-red-500 py-4">Failed to load downtime records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "downtime-table");
	}
}
function updateDowntimeStats(downtimes) {
	const totalRecords = downtimes.length;
	const totalMinutes = downtimes.reduce((sum, downtime) => sum + (downtime.duration_minutes || 0), 0);
	const dayDowntime = downtimes.filter((d) => d.shift_id === 1).reduce((sum, d) => sum + (d.duration_minutes || 0), 0);
	const nightDowntime = downtimes.filter((d) => d.shift_id === 2).reduce((sum, d) => sum + (d.duration_minutes || 0), 0);

	document.getElementById("total-records").textContent = totalRecords;
	document.getElementById("total-downtime").textContent = formatDowntime(totalMinutes);
	document.getElementById("day-downtime").textContent = formatDowntime(dayDowntime);
	document.getElementById("night-downtime").textContent = formatDowntime(nightDowntime);
}

function renderDowntimes(downtimesToRender) {
	const tbody = document.getElementById("downtime-table-body");
	if (!downtimesToRender || downtimesToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="8" class="text-center text-gray-500 py-4">No downtime records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	downtimesToRender.forEach((downtime) => {
		const shift = shifts.find((s) => s.id === downtime.shift_id);
		const reason = downtimeReasons.find((r) => r.id === downtime.downtime_reason_id);
		const createdBy = users.find((u) => u.id === downtime.created_by);

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
			<td class="py-3 px-4">${escapeHtml(shift?.name || "Unknown")}</td>
			<td class="py-3 px-4">${formatDateTime(downtime.start_time)}</td>
			<td class="py-3 px-4">${formatDateTime(downtime.end_time)}</td>
            <td class="py-3 px-4 text-center">${formatDowntime(downtime.duration_minutes || 0)}</td>
			<td class="py-3 px-4">${escapeHtml(reason?.name || "Unknown")}</td>
			<td class="py-3 px-4">${escapeHtml(createdBy?.full_name || "System")}</td>
			<td class="py-3 px-4">${formatDate(downtime.created_at)}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2">
					<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${downtime.id}">
						<i class="fas fa-edit"></i>
					</button>
					<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${downtime.id}">
						<i class="fas fa-trash"></i>
					</button>
				</div>
			</td>
		`;

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => editDowntime(btn.dataset.id));
	});
	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			deleteDowntime(btn.dataset.id);
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
	showLoading(true, "downtime-table");

	try {
		const params = new URLSearchParams();
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const reasonFilter = document.getElementById("filter-reason").value;
		const userFilter = document.getElementById("filter-user").value;

		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (reasonFilter) params.append("downtime_reason_id", reasonFilter);
		if (userFilter) params.append("created_by", userFilter);
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/downtimes/filter?${params}`);
		const result = await handleApiResponse(response);

		downtimes = result.data;
		totalCount = result.total_count;
		updateDowntimeStats(downtimes);
		renderDowntimes(downtimes);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		showLoading(false, "downtime-table");
	}
}

function clearFilters() {
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-reason").value = "";
	document.getElementById("filter-user").value = "";

	currentPage = 1;
	applyFilters();
}

function openModal(downtimeId = null) {
	const modal = document.getElementById("downtime-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("downtime-form");

	if (downtimeId) {
		title.textContent = "Edit Downtime";
		const downtime = downtimes.find((d) => d.id === parseInt(downtimeId));
		if (downtime) {
			populateForm(downtime);
			const downtimeTime = new Date(downtime.start_time);
			const now = new Date();
			const hoursDiff = (now - downtimeTime) / (1000 * 60 * 60);

			if (hoursDiff > 24) {
				document.getElementById("shift").disabled = true;
				document.getElementById("downtime-reason").disabled = true;
				document.getElementById("start-time").disabled = true;
				document.getElementById("end-time").disabled = true;
				document.getElementById("duration").disabled = true;
				document.querySelector('button[type="submit"]').disabled = true;
				document.querySelector('button[type="submit"]').innerHTML = "Editing Disabled (Over 24 hours)";
			}
		}
	} else {
		title.textContent = "Add Downtime";
		form.reset();
		document.getElementById("downtime-id").value = "";
		enableDowntimeForm();
	}

	modal.style.display = "flex";
}

function enableDowntimeForm() {
	document.getElementById("shift").disabled = false;
	document.getElementById("downtime-reason").disabled = false;
	document.getElementById("start-time").disabled = false;
	document.getElementById("end-time").disabled = false;
	document.getElementById("duration").disabled = false;
	const submitBtn = document.querySelector('button[type="submit"]');
	submitBtn.disabled = false;
	submitBtn.innerHTML = "Save";
}

function closeModal() {
	document.getElementById("downtime-modal").style.display = "none";
	enableDowntimeForm();
}

function populateForm(downtime) {
	document.getElementById("downtime-id").value = downtime.id;
	document.getElementById("shift").value = downtime.shift_id;
	document.getElementById("downtime-reason").value = downtime.downtime_reason_id;
	document.getElementById("start-time").value = formatDateTimeLocal(downtime.start_time);
	document.getElementById("end-time").value = formatDateTimeLocal(downtime.end_time);
	document.getElementById("duration").value = downtime.duration_minutes || 0;
}

function calculateDuration() {
	const startTime = document.getElementById("start-time").value;
	const endTime = document.getElementById("end-time").value;

	if (startTime && endTime) {
		const start = new Date(startTime);
		const end = new Date(endTime);
		const duration = Math.max(0, Math.round((end - start) / (1000 * 60)));
		document.getElementById("duration").value = duration;
	} else {
		document.getElementById("duration").value = "";
	}
}

function handleFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const downtimeId = document.getElementById("downtime-id").value;
	const formData = {
		shift_id: parseInt(document.getElementById("shift").value),
		downtime_reason_id: parseInt(document.getElementById("downtime-reason").value),
		start_time: document.getElementById("start-time").value,
		end_time: document.getElementById("end-time").value,
		duration_minutes: parseInt(document.getElementById("duration").value),
	};

	if (downtimeId) {
		formData.id = parseInt(downtimeId);
		updateDowntime(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createDowntime(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function createDowntime(downtimeData) {
	try {
		const response = await fetch("/api/downtimes/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(downtimeData),
		});
		await handleApiResponse(response);

		showNotification("Downtime record created successfully!", "success");
		closeModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateDowntime(downtimeData) {
	try {
		const response = await fetch("/api/downtimes/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(downtimeData),
		});
		await handleApiResponse(response);

		showNotification("Downtime record updated successfully!", "success");
		closeModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editDowntime(downtimeId) {
	openModal(downtimeId);
}

async function deleteDowntime(downtimeId) {
	if (!confirm("Are you sure you want to delete this downtime record?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${downtimeId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/downtimes/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(downtimeId) }),
		});
		await handleApiResponse(response);

		showNotification("Downtime record deleted successfully!", "success");
		await applyFilters();
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
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const reasonFilter = document.getElementById("filter-reason").value;
		const userFilter = document.getElementById("filter-user").value;

		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (reasonFilter) params.append("downtime_reason_id", reasonFilter);
		if (userFilter) params.append("created_by", userFilter);

		const response = await fetch(`/api/downtimes/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredDowntimes = result.data;

		if (filteredDowntimes.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredDowntimes.map((downtime) => {
			const shift = shifts.find((s) => s.id === downtime.shift_id);
			const reason = downtimeReasons.find((r) => r.id === downtime.downtime_reason_id);
			const createdBy = users.find((u) => u.id === downtime.created_by);

			return {
				Shift: shift?.name || "Unknown",
				"Start Time": formatDateTime(downtime.start_time),
				"End Time": formatDateTime(downtime.end_time),
				Duration: formatDowntime(downtime.duration_minutes || 0),
				Reason: reason?.name || "Unknown",
				"Created By": createdBy?.full_name || "System",
				"Created At": formatDate(downtime.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Downtime Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "downtime_records.xlsx");

		showNotification("Downtime records exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", applyFilters);
	document.getElementById("clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-downtime-btn").addEventListener("click", () => openModal());
	document.getElementById("close-modal").addEventListener("click", closeModal);
	document.getElementById("cancel-btn").addEventListener("click", closeModal);
	document.getElementById("downtime-form").addEventListener("submit", handleFormSubmit);

	document.getElementById("start-time").addEventListener("change", calculateDuration);
	document.getElementById("end-time").addEventListener("change", calculateDuration);

	const exportBtn = document.getElementById("export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}

	document.getElementById("per-page").addEventListener("change", function () {
		itemsPerPage = parseInt(this.value);
		currentPage = 1;
		applyFilters();
	});

	setupShiftTimeRestrictions();
}
