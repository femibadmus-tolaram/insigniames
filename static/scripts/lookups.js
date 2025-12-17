/** @format */
let lookups = [];
let currentPage = 1;
let itemsPerPage = 10;

const lookupTypes = {
	shifts: "Shifts",
	colours: "Colours",
	"solvent-types": "Solvent Types",
	"scrap-types": "Scrap Types",
	"po-codes": "Material Description CODE",
	"downtime-reasons": "Downtime Reasons",
	"flag-reasons": "Flag Reasons",
};

const colorSuggestions = {
	Red: "#dc2626",
	Blue: "#2563eb",
	Green: "#16a34a",
	Yellow: "#ca8a04",
	Black: "#000000",
	White: "#ffffff",
	Orange: "#ea580c",
	Purple: "#9333ea",
	Pink: "#db2777",
	Brown: "#92400e",
	Gray: "#6b7280",
	Cyan: "#0891b2",
};

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadLookups();
	setupEventListeners();
}

function showLoading(show) {
	const loadingMessage = document.getElementById("loading-message");
	const table = document.getElementById("lookups-table");

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

async function loadLookups() {
	showLoading(true);
	try {
		const promises = Object.keys(lookupTypes).map((type) => fetch(`/api/lookups/${type}`).then((res) => res.json()));

		const results = await Promise.all(promises);
		lookups = [];

		Object.keys(lookupTypes).forEach((type, index) => {
			const typeData = results[index];
			typeData.forEach((item) => {
				lookups.push({
					...item,
					lookup_type: type,
					lookup_type_name: lookupTypes[type],
				});
			});
		});

		updateLookupStats(lookups);
		renderLookups(lookups);
	} catch (error) {
		document.getElementById("lookups-table-body").innerHTML =
			'<tr><td colspan="3" class="text-center text-red-500 py-4">Failed to load lookup records</td></tr>';
		showNotification("Failed to load lookup records. Please try again later.", "error");
	} finally {
		showLoading(false);
	}
}

function updateLookupStats(lookups) {
	const totalLookups = lookups.length;
	const totalTypes = new Set(lookups.map((l) => l.lookup_type)).size;

	document.getElementById("total-lookups").textContent = totalLookups;
	document.getElementById("total-types").textContent = totalTypes;
}

function renderLookups(lookupsToRender) {
	const tbody = document.getElementById("lookups-table-body");
	if (!lookupsToRender || lookupsToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="3" class="text-center text-gray-500 py-4">No lookup records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	lookupsToRender.forEach((lookup) => {
		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
            <td class="py-3 px-4">
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    ${escapeHtml(lookup.lookup_type_name)}
                </span>
            </td>
            <td class="py-3 px-4">
                <div class="flex items-center gap-2">
                    ${
						lookup.lookup_type === "colours" && lookup.color_code
							? `<div class="w-4 h-4 rounded border border-gray-300" style="background-color: ${lookup.color_code}"></div>`
							: ""
					}
                    ${escapeHtml(lookup.name)}
                </div>
            </td>
            <td class="py-3 px-4">
                <div class="flex gap-2">
                    <button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${lookup.id}" data-type="${lookup.lookup_type}">
                        <i class="fas fa-edit"></i>
                    </button>
                    <button class="text-red-600 hover:text-red-800 delete-btn" data-id="${lookup.id}" data-type="${lookup.lookup_type}">
                        <i class="fas fa-trash"></i>
                    </button>
                </div>
            </td>
        `;

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => {
			const type = btn.dataset.type;
			const id = btn.dataset.id;
			editLookup(id, type);
		});
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const type = this.dataset.type;
			const id = this.dataset.id;
			deleteLookup(id, type);
		});
	});
}

function applyFilters() {
	try {
		const typeFilter = document.getElementById("filter-type").value;

		let filteredLookups = lookups;

		if (typeFilter) {
			filteredLookups = filteredLookups.filter((lookup) => lookup.lookup_type === typeFilter);
		}

		updateLookupStats(filteredLookups);
		renderLookups(filteredLookups);
	} catch (error) {
		showNotification("Failed to apply filters. Please try again.", "error");
	}
}

function clearFilters() {
	document.getElementById("filter-type").value = "";
	currentPage = 1;
	loadLookups();
}

function openModal(lookupId = null, lookupType = null) {
	const modal = document.getElementById("lookup-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("lookup-form");
	const colorPicker = document.getElementById("color-picker");
	const colorSuggestionsContainer = document.getElementById("color-suggestions");

	// Show/hide color picker based on type
	const typeSelect = document.getElementById("type");
	typeSelect.addEventListener("change", toggleColorPicker);

	if (lookupId && lookupType) {
		title.textContent = "Edit Lookup";
		const lookup = lookups.find((l) => l.id === parseInt(lookupId) && l.lookup_type === lookupType);
		if (lookup) {
			populateForm(lookup);
		}
	} else {
		title.textContent = "Add Lookup";
		form.reset();
		document.getElementById("lookup-id").value = "";
		document.getElementById("lookup-type").value = "";
		toggleColorPicker();
	}

	modal.style.display = "flex";
}

function toggleColorPicker() {
	const type = document.getElementById("type").value;
	const colorPicker = document.getElementById("color-picker");
	const colorSuggestions = document.getElementById("color-suggestions");

	if (type === "colours") {
		colorPicker.style.display = "block";
		colorSuggestions.style.display = "block";
		renderColorSuggestions();
	} else {
		colorPicker.style.display = "none";
		colorSuggestions.style.display = "none";
	}
}

function renderColorSuggestions() {
	const container = document.getElementById("color-suggestions");
	container.innerHTML = '<div class="text-sm text-gray-600 mb-2">Quick picks:</div><div class="flex flex-wrap gap-2">';

	Object.entries(colorSuggestions).forEach(([name, color]) => {
		container.innerHTML += `
            <button type="button" class="color-suggestion-btn w-8 h-8 rounded border border-gray-300 hover:scale-110 transition-transform" 
                    style="background-color: ${color}" title="${name}"
                    onclick="selectColor('${name}', '${color}')">
            </button>
        `;
	});

	container.innerHTML += "</div>";
}

function selectColor(name, color) {
	document.getElementById("name").value = name;
	document.getElementById("color-code").value = color;
	document.getElementById("color-preview").style.backgroundColor = color;
}

function closeModal() {
	document.getElementById("lookup-modal").style.display = "none";
}

function populateForm(lookup) {
	document.getElementById("lookup-id").value = lookup.id;
	document.getElementById("lookup-type").value = lookup.lookup_type;
	document.getElementById("type").value = lookup.lookup_type;
	document.getElementById("name").value = lookup.name;

	if (lookup.lookup_type === "colours" && lookup.color_code) {
		document.getElementById("color-code").value = lookup.color_code;
		document.getElementById("color-preview").style.backgroundColor = lookup.color_code;
	}

	toggleColorPicker();
}

function checkDuplicateLookup(type, name, excludeId = null) {
	return lookups.some((lookup) => lookup.lookup_type === type && lookup.name.toLowerCase() === name.toLowerCase() && lookup.id !== excludeId);
}

function handleFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const lookupId = document.getElementById("lookup-id").value;
	const lookupType = document.getElementById("lookup-type").value;
	const selectedType = document.getElementById("type").value;
	const name = document.getElementById("name").value.trim();
	const colorCode = document.getElementById("color-code").value;

	// Check for duplicates
	const finalType = lookupType || selectedType;
	if (checkDuplicateLookup(finalType, name, lookupId ? parseInt(lookupId) : null)) {
		showNotification(`A ${lookupTypes[finalType]} with name "${name}" already exists!`, "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	const formData = {
		name: name,
	};

	// Add color code for colours
	if (finalType === "colours" && colorCode) {
		formData.color_code = colorCode;
	}

	const endpoint = lookupId ? "update" : "create";
	const apiType = lookupType || selectedType;

	if (lookupId) {
		formData.id = parseInt(lookupId);
		updateLookup(apiType, formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createLookup(apiType, formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function createLookup(type, lookupData) {
	try {
		const response = await fetch(`/api/lookups/${type}/create`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(lookupData),
		});

		if (!response.ok) {
			const errorData = await response.json();
			throw new Error(errorData.message || `Failed to create ${lookupTypes[type]} record`);
		}

		showNotification(`${lookupTypes[type]} record created successfully!`, "success");
		closeModal();
		await loadLookups();
	} catch (error) {
		showNotification(error.message || `Error creating ${lookupTypes[type]} record. Please try again.`, "error");
	}
}

async function updateLookup(type, lookupData) {
	try {
		const response = await fetch(`/api/lookups/${type}/update`, {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(lookupData),
		});

		if (!response.ok) {
			const errorData = await response.json();
			throw new Error(errorData.message || `Failed to update ${lookupTypes[type]} record`);
		}

		showNotification(`${lookupTypes[type]} record updated successfully!`, "success");
		closeModal();
		await loadLookups();
	} catch (error) {
		showNotification(error.message || `Error updating ${lookupTypes[type]} record. Please try again.`, "error");
	}
}

function editLookup(lookupId, lookupType) {
	openModal(lookupId, lookupType);
}

async function deleteLookup(lookupId, lookupType) {
	if (!confirm("Are you sure you want to delete this lookup record?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${lookupId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch(`/api/lookups/${lookupType}/delete`, {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(lookupId) }),
		});

		if (!response.ok) {
			let errorText;
			const text = await response.text();
			try {
				errorText = JSON.parse(text).message;
			} catch {
				errorText = text;
			}
			throw new Error(errorText);
		}

		showNotification(`${lookupTypes[lookupType]} record deleted successfully!`, "success");
		await loadLookups();
	} catch (error) {
		showNotification(error.message || `Error deleting ${lookupTypes[lookupType]} record. Please try again.`, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

async function exportToExcel() {
	const exportBtn = document.getElementById("export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const filteredLookups = getFilteredLookups();

		if (filteredLookups.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredLookups.map((lookup) => ({
			Type: lookup.lookup_type_name,
			Name: lookup.name,
			"Color Code": lookup.color_code || "N/A",
		}));

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Lookup Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "lookup_records.xlsx");

		showNotification("Lookup records exported successfully!", "success");
	} catch (error) {
		showNotification("Error exporting lookup records. Please try again.", "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function getFilteredLookups() {
	const typeFilter = document.getElementById("filter-type").value;

	let filteredLookups = lookups;

	if (typeFilter) {
		filteredLookups = filteredLookups.filter((lookup) => lookup.lookup_type === typeFilter);
	}

	return filteredLookups;
}

function saveAsExcel(buffer, filename) {
	const blob = new Blob([buffer], { type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" });
	const url = URL.createObjectURL(blob);
	const a = document.createElement("a");
	a.href = url;
	a.download = filename;
	document.body.appendChild(a);
	a.click();
	setTimeout(() => {
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}, 100);
}

function setupEventListeners() {
	document.getElementById("filter-type").addEventListener("change", applyFilters);
	document.getElementById("add-lookup-btn").addEventListener("click", () => openModal());
	document.getElementById("close-modal").addEventListener("click", closeModal);
	document.getElementById("cancel-btn").addEventListener("click", closeModal);
	document.getElementById("lookup-form").addEventListener("submit", handleFormSubmit);

	// Color picker events
	document.getElementById("color-code").addEventListener("input", function () {
		document.getElementById("color-preview").style.backgroundColor = this.value;
	});

	const exportBtn = document.getElementById("export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}
}
