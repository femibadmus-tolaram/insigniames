/** @format */

let rolls = [];
let flagReasons = [];
let users = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadFilterOptions();
	await loadRolls();
	setupEventListeners();
}

async function loadFilterOptions() {
	try {
		const [reasonsResponse, usersResponse] = await Promise.all([
			fetch("/api/lookups/flag-reasons").then(handleApiResponse),
			fetch("/api/users").then(handleApiResponse),
		]);

		flagReasons = reasonsResponse;
		users = usersResponse;

		populateSelect("filter-flag-reason", flagReasons, "name", "All Reasons");
		populateSelect("filter-user", users, "full_name", "All Users");
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadRolls() {
	showLoading(true, "rolls-table");
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/rolls/filter?${params}`);
		const result = await handleApiResponse(response);

		rolls = result.data;
		totalCount = result.total_count;
		updateRollStats(rolls);
		renderRolls(rolls);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		document.getElementById("rolls-table-body").innerHTML =
			'<tr><td colspan="8" class="text-center text-red-500 py-4">Failed to load roll records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "rolls-table");
	}
}

function updateRollStats(rolls) {
	const totalRecords = rolls.length;
	const completedRolls = rolls.filter((roll) => roll.final_weight > 0).length;
	const pendingRolls = rolls.filter((roll) => roll.final_weight === 0).length;
	const flaggedRolls = rolls.filter((roll) => roll.number_of_flags > 0).length;

	document.getElementById("total-rolls").textContent = totalRecords;
	document.getElementById("completed-rolls").textContent = completedRolls;
	document.getElementById("pending-rolls").textContent = pendingRolls;
	document.getElementById("flagged-rolls").textContent = flaggedRolls;
}

function renderRolls(rollsToRender) {
	const tbody = document.getElementById("rolls-table-body");
	if (!rollsToRender || rollsToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="8" class="text-center text-gray-500 py-4">No roll records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	rollsToRender.forEach((roll) => {
		const reason = flagReasons.find((r) => r.id === roll.flag_reason_id);
		const createdBy = users.find((u) => u.id === roll.created_by);
		const isEditable = roll.final_weight === 0;
		const isDeletable = roll.final_weight === 0;

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
			<td class="py-3 px-4 font-medium">${escapeHtml(roll.output_roll_no)}</td>
			<td class="py-3 px-4">
				<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
					roll.final_weight === 0
						? "bg-yellow-100 text-yellow-800"
						: roll.number_of_flags > 0
						? "bg-red-100 text-red-800"
						: "bg-green-100 text-green-800"
				}">
					${roll.final_weight === 0 ? "Pending" : roll.number_of_flags > 0 ? "Flagged" : "Completed"}
				</span>
			</td>
			<td class="py-3 px-4 text-center">${formatWeight(roll.final_weight || 0)}</td>
			<td class="py-3 px-4 text-center">${formatMeter(roll.final_meter || 0)}</td>
			<td class="py-3 px-4 text-center">${roll.job_id}</td>
			<td class="py-3 px-4 text-center">${roll.number_of_flags || 0}</td>
			<td class="py-3 px-4">${formatDateTime(roll.created_at)}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2">
					<button class="${isEditable ? "text-blue-600 hover:text-blue-800" : "text-gray-400 cursor-not-allowed"} edit-btn" data-id="${roll.id}" ${
			!isEditable ? "disabled" : ""
		}>
						<i class="fas fa-edit"></i>
					</button>
					<button class="${isDeletable ? "text-red-600 hover:text-red-800" : "text-gray-400 cursor-not-allowed"} delete-btn" data-id="${roll.id}" ${
			!isDeletable ? "disabled" : ""
		}>
						<i class="fas fa-trash"></i>
					</button>
				</div>
			</td>
		`;

		if (roll.flag_reason_id) {
			const reason = flagReasons.find((r) => r.id === roll.flag_reason_id);
			if (reason) {
				row.setAttribute("title", `Flag Reason: ${reason.name}`);
				row.classList.add("cursor-help");
			}
		}

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn:not([disabled])").forEach((btn) => {
		btn.addEventListener("click", () => editRoll(btn.dataset.id));
	});
	document.querySelectorAll(".delete-btn:not([disabled])").forEach((btn) => {
		btn.addEventListener("click", function () {
			deleteRoll(btn.dataset.id);
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
	showLoading(true, "rolls-table");

	try {
		const params = new URLSearchParams();
		const searchTerm = document.getElementById("filter-search").value;
		const statusFilter = document.getElementById("filter-status").value;
		const reasonFilter = document.getElementById("filter-flag-reason").value;
		const userFilter = document.getElementById("filter-user").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;

		if (searchTerm) {
			if (!isNaN(searchTerm) && searchTerm.trim() !== "") {
				params.append("job_id", searchTerm);
			} else {
				params.append("output_roll_no", searchTerm);
			}
		}

		if (statusFilter) params.append("status", statusFilter);

		if (reasonFilter) params.append("flag_reason_id", reasonFilter);
		if (userFilter) params.append("created_by", userFilter);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/rolls/filter?${params}`);
		const result = await handleApiResponse(response);

		rolls = result.data;
		totalCount = result.total_count;
		updateRollStats(rolls);
		renderRolls(rolls);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		showLoading(false, "rolls-table");
	}
}

function clearFilters() {
	document.getElementById("filter-search").value = "";
	document.getElementById("filter-status").value = "";
	document.getElementById("filter-flag-reason").value = "";
	document.getElementById("filter-user").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";

	currentPage = 1;
	applyFilters();
}

function openModal(rollId = null) {
	const modal = document.getElementById("roll-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("roll-form");

	if (rollId) {
		title.textContent = "Update Roll";
		const roll = rolls.find((r) => r.id === parseInt(rollId));
		if (roll) {
			populateForm(roll);
		}
	} else {
		title.textContent = "Add Roll";
		form.reset();
		document.getElementById("roll-id").value = "";
	}

	modal.style.display = "flex";
}

function closeModal() {
	document.getElementById("roll-modal").style.display = "none";
}

function populateForm(roll) {
	document.getElementById("roll-id").value = roll.id;
	document.getElementById("final-weight").value = roll.final_weight || "";
}

function handleFormSubmit(e) {
	e.preventDefault();

	const rollId = document.getElementById("roll-id").value;
	const roll = rolls.find((r) => r.id === parseInt(rollId));

	if (roll && roll.final_weight > 0) {
		showNotification("Only pending rolls can be updated", "warning");
		return;
	}

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const formData = {
		id: parseInt(rollId),
		final_weight: parseFloat(document.getElementById("final-weight").value),
	};

	updateRoll(formData).finally(() => {
		setButtonLoading(submitBtn, false);
	});
}

async function updateRoll(rollData) {
	try {
		const response = await fetch("/api/rolls/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(rollData),
		});
		await handleApiResponse(response);

		showNotification("Roll updated successfully!", "success");
		closeModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editRoll(rollId) {
	const roll = rolls.find((r) => r.id === parseInt(rollId));

	if (roll.final_weight > 0 && roll.number_of_flags === 0) {
		showNotification("Completed rolls cannot be edited", "warning");
		return;
	}

	if (roll.final_weight > 0 && roll.number_of_flags > 0) {
		showNotification("Flagged rolls cannot be edited", "warning");
		return;
	}

	openModal(rollId);
}

async function deleteRoll(rollId) {
	if (!confirm("Are you sure you want to delete this roll?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${rollId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/rolls/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(rollId) }),
		});
		await handleApiResponse(response);

		showNotification("Roll deleted successfully!", "success");
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
		const searchTerm = document.getElementById("filter-search").value;
		const statusFilter = document.getElementById("filter-status").value;
		const reasonFilter = document.getElementById("filter-flag-reason").value;
		const userFilter = document.getElementById("filter-user").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;

		if (searchTerm) {
			if (!isNaN(searchTerm) && searchTerm.trim() !== "") {
				params.append("job_id", searchTerm);
			} else {
				params.append("output_roll_no", searchTerm);
			}
		}

		if (statusFilter) {
			if (statusFilter === "pending") {
				params.append("final_weight", "0");
			} else if (statusFilter === "flagged") {
				params.append("number_of_flags_gt", "0");
			} else if (statusFilter === "completed") {
				params.append("final_weight_gt", "0");
				params.append("number_of_flags", "0");
			}
		}

		if (reasonFilter) params.append("flag_reason_id", reasonFilter);
		if (userFilter) params.append("created_by", userFilter);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);

		const response = await fetch(`/api/rolls/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredRolls = result.data;

		if (filteredRolls.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredRolls.map((roll) => {
			const reason = flagReasons.find((r) => r.id === roll.flag_reason_id);
			const createdBy = users.find((u) => u.id === roll.created_by);

			return {
				"Output Roll No": roll.output_roll_no,
				Status: roll.final_weight === 0 ? "Pending" : roll.number_of_flags > 0 ? "Flagged" : "Completed",
				Weight: formatWeight(roll.final_weight || 0),
				Meters: formatMeter(roll.final_meter || 0),
				"Job ID": roll.job_id,
				"Flag Reason": reason?.name || "-",
				"Flag Count": roll.number_of_flags || 0,
				"Created By": createdBy?.full_name || "System",
				"Created At": formatDateTime(roll.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Roll Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "roll_records.xlsx");

		showNotification("Roll records exported successfully!", "success");
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
	document.getElementById("roll-form").addEventListener("submit", handleFormSubmit);

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
