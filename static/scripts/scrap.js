/** @format */
let scraps = [];
let scrapTypes = [];
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
	await loadScraps();
	setupEventListeners();
}

async function loadFilterOptions() {
	try {
		const [typesResponse, usersResponse] = await Promise.all([
			fetch("/api/lookups/scrap-types").then(handleApiResponse),
			fetch("/api/users").then(handleApiResponse),
		]);

		scrapTypes = typesResponse;
		users = usersResponse;

		populateSelect("filter-type", scrapTypes, "name", "All Types");
		populateSelect("filter-user", users, "full_name", "All Users");
		populateSelect("scrap-type", scrapTypes, "name", "Select Scrap Type");
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function showLoading(show) {
	const loadingMessage = document.getElementById("loading-message");
	const table = document.getElementById("scrap-table");

	if (show) {
		loadingMessage.style.display = "block";
		table.style.display = "none";
	} else {
		loadingMessage.style.display = "none";
		table.style.display = "table";
	}
}

function setButtonLoading(button, loading) {
	if (loading) {
		button.disabled = true;
		const originalHTML = button.innerHTML;
		button.setAttribute("data-original-html", originalHTML);
		button.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Loading...';
	} else {
		button.disabled = false;
		const originalHTML = button.getAttribute("data-original-html");
		if (originalHTML) {
			button.innerHTML = originalHTML;
		}
	}
}

async function loadScraps() {
	showLoading(true, "scrap-table");
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/scraps/filter?${params}`);
		const result = await handleApiResponse(response);

		scraps = result.data;
		totalCount = result.total_count;
		updateScrapStats(scraps);
		renderScraps(scraps);
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		document.getElementById("scrap-table-body").innerHTML =
			'<tr><td colspan="8" class="text-center text-red-500 py-4">Failed to load scrap records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false);
	}
}

function updateScrapStats(scraps) {
	const totalRecords = scraps.length;
	const totalWeight = scraps.reduce((sum, scrap) => sum + (scrap.weight_kg || 0), 0);
	const dayWeight = scraps.filter((s) => s.shift_id === 1).reduce((sum, s) => sum + (s.weight_kg || 0), 0);
	const nightWeight = scraps.filter((s) => s.shift_id === 2).reduce((sum, s) => sum + (s.weight_kg || 0), 0);

	document.getElementById("total-records").textContent = totalRecords;
	document.getElementById("total-weight").textContent = `${formatWeight(totalWeight)}`;
	document.getElementById("day-weight").textContent = `${formatWeight(dayWeight)}`;
	document.getElementById("night-weight").textContent = `${formatWeight(nightWeight)}`;
}

function renderScraps(scrapsToRender) {
	const tbody = document.getElementById("scrap-table-body");
	if (!scrapsToRender || scrapsToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="8" class="text-center text-gray-500 py-4">No scrap records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	scrapsToRender.forEach((scrap) => {
		const shift = shifts.find((s) => s.id === scrap.shift_id);
		const type = scrapTypes.find((t) => t.id === scrap.scrap_type_id);
		const createdBy = users.find((u) => u.id === scrap.created_by);

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
            <td class="py-3 px-4">${escapeHtml(shift?.name || "Unknown")}</td>
            <td class="py-3 px-4">${formatDateTime(scrap.time)}</td>
            <td class="py-3 px-4">${escapeHtml(type?.name || "Unknown")}</td>
            <td class="py-3 px-4 text-center">${(scrap.weight_kg || 0).toFixed(2)}</td>
            <!--<td class="py-3 px-4">${escapeHtml(scrap.notes || "N/A")}</td>-->
            <td class="py-3 px-4">${escapeHtml(createdBy?.full_name || "System")}</td>
            <!--<td class="py-3 px-4">${formatDate(scrap.created_at)}</td>-->
            <td class="py-3 px-4">
                <div class="flex gap-2">
                    <button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${scrap.id}">
                        <i class="fas fa-edit"></i>
                    </button>
                    <button class="text-red-600 hover:text-red-800 delete-btn" data-id="${scrap.id}">
                        <i class="fas fa-trash"></i>
                    </button>
                </div>
            </td>
        `;

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => editScrap(btn.dataset.id));
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			deleteScrap(this.dataset.id);
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
	showLoading(true, "scrap-table");

	try {
		const params = new URLSearchParams();
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const typeFilter = document.getElementById("filter-type").value;
		const userFilter = document.getElementById("filter-user").value;

		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (typeFilter) params.append("scrap_type_id", typeFilter);
		if (userFilter) params.append("created_by", userFilter);
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/scraps/filter?${params}`);
		const result = await handleApiResponse(response);

		scraps = result.data;
		totalCount = result.total_count;
		updateScrapStats(scraps);
		renderScraps(scraps);
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		showLoading(false);
	}
}

function clearFilters() {
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-type").value = "";
	document.getElementById("filter-user").value = "";

	currentPage = 1;
	applyFilters();
}

function openModal(scrapId = null) {
	const modal = document.getElementById("scrap-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("scrap-form");

	if (scrapId) {
		title.textContent = "Edit Scrap";
		const scrap = scraps.find((s) => s.id === parseInt(scrapId));
		if (scrap) {
			populateForm(scrap);
			const scrapTime = new Date(scrap.time);
			const now = new Date();
			const hoursDiff = (now - scrapTime) / (1000 * 60 * 60);

			if (hoursDiff > 24) {
				document.getElementById("shift").disabled = true;
				document.getElementById("scrap-type").disabled = true;
				document.getElementById("time").disabled = true;
				document.getElementById("weight").disabled = true;
				document.getElementById("notes").disabled = true;
				document.querySelector('button[type="submit"]').disabled = true;
				document.querySelector('button[type="submit"]').innerHTML = "Editing Disabled (Over 24 hours)";
			}
		}
	} else {
		title.textContent = "Add Scrap";
		form.reset();
		document.getElementById("scrap-id").value = "";
		enableForm();
	}

	modal.style.display = "flex";
}

function enableForm() {
	document.getElementById("shift").disabled = false;
	document.getElementById("scrap-type").disabled = false;
	document.getElementById("time").disabled = false;
	document.getElementById("weight").disabled = false;
	document.getElementById("notes").disabled = false;
	const submitBtn = document.querySelector('button[type="submit"]');
	submitBtn.disabled = false;
	submitBtn.innerHTML = "Save";
}

function closeModal() {
	document.getElementById("scrap-modal").style.display = "none";
	enableForm();
}

function populateForm(scrap) {
	document.getElementById("scrap-id").value = scrap.id;
	document.getElementById("shift").value = scrap.shift_id;
	document.getElementById("scrap-type").value = scrap.scrap_type_id;
	document.getElementById("time").value = formatDateTimeLocal(scrap.time);
	document.getElementById("weight").value = scrap.weight_kg || "";
	document.getElementById("notes").value = scrap.notes || "";
}

function handleFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const scrapId = document.getElementById("scrap-id").value;
	const formData = {
		shift_id: parseInt(document.getElementById("shift").value),
		scrap_type_id: parseInt(document.getElementById("scrap-type").value),
		time: document.getElementById("time").value,
		weight_kg: parseFloat(document.getElementById("weight").value),
		notes: document.getElementById("notes").value,
	};

	if (scrapId) {
		formData.id = parseInt(scrapId);
		updateScrap(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createScrap(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function createScrap(scrapData) {
	try {
		const response = await fetch("/api/scraps/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(scrapData),
		});
		await handleApiResponse(response);

		showNotification("Scrap record created successfully!", "success");
		closeModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateScrap(scrapData) {
	try {
		const response = await fetch("/api/scraps/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(scrapData),
		});
		await handleApiResponse(response);

		showNotification("Scrap record updated successfully!", "success");
		closeModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editScrap(scrapId) {
	openModal(scrapId);
}

async function deleteScrap(scrapId) {
	if (!confirm("Are you sure you want to delete this scrap record?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${scrapId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/scraps/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(scrapId) }),
		});
		await handleApiResponse(response);

		showNotification("Scrap record deleted successfully!", "success");
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
		const typeFilter = document.getElementById("filter-type").value;
		const userFilter = document.getElementById("filter-user").value;

		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (typeFilter) params.append("scrap_type_id", typeFilter);
		if (userFilter) params.append("created_by", userFilter);

		const response = await fetch(`/api/scraps/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredScraps = result.data;

		if (filteredScraps.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredScraps.map((scrap) => {
			const shift = shifts.find((s) => s.id === scrap.shift_id);
			const type = scrapTypes.find((t) => t.id === scrap.scrap_type_id);
			const createdBy = users.find((u) => u.id === scrap.created_by);

			return {
				Shift: shift?.name || "Unknown",
				Time: formatDateTime(scrap.time),
				"Scrap Type": type?.name || "Unknown",
				"Weight (kg)": (scrap.weight_kg || 0).toFixed(2),
				Notes: scrap.notes || "N/A",
				"Created By": createdBy?.full_name || "System",
				"Created At": formatDate(scrap.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Scrap Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "scrap_records.xlsx");

		showNotification("Scrap records exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", applyFilters);
	document.getElementById("clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-scrap-btn").addEventListener("click", () => openModal());
	document.getElementById("close-modal").addEventListener("click", closeModal);
	document.getElementById("cancel-btn").addEventListener("click", closeModal);
	document.getElementById("scrap-form").addEventListener("submit", handleFormSubmit);

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
