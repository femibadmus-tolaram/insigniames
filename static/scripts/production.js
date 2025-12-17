/** @format */

let currentUser = null;
let machines = [];
let processOrders = {};
let flagReasons = [];
let currentJob = null;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadCurrentUser();
	await loadMachines();
	await loadFlagReasons();
	setupEventListeners();
}

async function loadCurrentUser() {
	try {
		const response = await fetch("/api/users/me");
		currentUser = await handleApiResponse(response);
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadMachines() {
	try {
		const response = await fetch("/api/machines");
		machines = await handleApiResponse(response);
		populateMachineSelect();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadProcessOrders() {
	const poSearchInput = document.getElementById("po-search");
	const poResultsDiv = document.getElementById("po-results");
	const shiftSelect = document.getElementById("shift-select");
	const dateSelect = document.getElementById("date-select");
	const machineSelect = document.getElementById("machine");
	const selectedMachineId = machineSelect.value;

	if (!selectedMachineId) {
		resetForm();
		poSearchInput.placeholder = "Select machine first";
		return;
	}

	if (!shiftSelect.value || !dateSelect.value) {
		resetForm();
		poSearchInput.placeholder = "Select shift and date first";
		return;
	}

	const selectedMachine = machines.find((m) => m.id == selectedMachineId);
	if (!selectedMachine || !currentUser || !currentUser.section_ids) {
		resetForm();
		poSearchInput.placeholder = "No section assigned";
		return;
	}

	let line = null;
	if (selectedMachine.name) {
		const lineMatch = selectedMachine.name.match(/LINE(\d+)/i);
		if (lineMatch) {
			line = lineMatch[1];
		}
	}

	const sectionIds = currentUser.section_ids.join(",");
	const url = `/api/process_order?section_ids=${sectionIds}${line ? `&line=${line}` : ""}&shift=${shiftSelect.value}&posting_date=${
		dateSelect.value
	}`;

	try {
		const response = await fetch(url);
		const result = await handleApiResponse(response);

		processOrders = {};

		if (result.data && Array.isArray(result.data)) {
			result.data.forEach((po) => {
				processOrders[po.process_order] = {
					process_order: po.process_order,
					description: po.description || "-",
					material_details: po.material_details || {},
				};
			});

			poSearchInput.disabled = false;
			poSearchInput.placeholder = "Search process order...";
		} else {
			resetForm();
			poSearchInput.placeholder = "No process orders found";
		}
	} catch (error) {
		showNotification(error.message, "error");
		resetForm();
		poSearchInput.placeholder = "Failed to load POs";
	}
}

function showAllPOs() {
	const poResultsDiv = document.getElementById("po-results");
	const poSearchInput = document.getElementById("po-search");

	poResultsDiv.innerHTML = "";

	if (Object.keys(processOrders).length === 0) {
		const noResult = document.createElement("div");
		noResult.className = "p-3 text-gray-500 text-center";
		noResult.textContent = "No process orders available";
		poResultsDiv.appendChild(noResult);
	} else {
		Object.values(processOrders).forEach((po) => {
			const div = document.createElement("div");
			div.className = "p-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100";
			div.dataset.poNumber = po.process_order;

			div.innerHTML = `
				<div class="font-medium text-gray-800">${po.process_order}</div>
				<div class="text-sm text-gray-600 truncate">${po.description || "-"}</div>
			`;

			div.addEventListener("click", function () {
				const poNumber = this.dataset.poNumber;
				const poData = processOrders[poNumber];

				document.getElementById("process-order").value = poNumber;
				poSearchInput.value = `${poNumber} - ${poData.description || ""}`;
				poResultsDiv.classList.add("hidden");

				displayJobDetails(poData);
				updateInputTitle(poData.material_details);

				document.getElementById("job-details").style.display = "block";
				document.getElementById("input-section").style.display = "block";

				document.querySelector("#input-form button[type='submit']").disabled = false;
			});

			poResultsDiv.appendChild(div);
		});
	}

	poResultsDiv.classList.remove("hidden");
}

function updateInputTitle(materialDetails) {
	const inputTitle = document.getElementById("input-title");
	if (!inputTitle) return;

	function extractSize(input) {
		const m = input.match(/(\d.*?MM)/i);
		return m ? m[1] : "";
	}

	let filmMaterial = "";
	const filmKeys = ["BOPP", "BOPA", "PET", "FILM"];

	for (const key of filmKeys) {
		if (materialDetails[key]) {
			filmMaterial = materialDetails[key];
			break;
		}
	}

	if (filmMaterial) {
		const dimensions = filmMaterial
			.split(",")
			.map((item) => item.trim())
			.filter((item) => item !== "");

		if (dimensions.length > 0) {
			inputTitle.textContent = `Currently Consuming (${dimensions.join(", ")})`;
		} else {
			inputTitle.textContent = "Currently Consuming (Input)";
		}
	} else {
		inputTitle.textContent = "Currently Consuming (Input)";
	}
}

async function loadFlagReasons() {
	try {
		const response = await fetch("/api/lookups/flag-reasons");
		flagReasons = await handleApiResponse(response);
		populateFlagReasons();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function populateMachineSelect() {
	const machineSelect = document.getElementById("machine");
	machineSelect.innerHTML = '<option value="">Select Machine</option>';

	if (currentUser && currentUser.id) {
		fetch(`/api/machines/filter?user_id=${currentUser.id}&per_page=100`)
			.then((response) => response.json())
			.then((result) => {
				const userMachines = result.data || result;

				userMachines.forEach((machine) => {
					const option = document.createElement("option");
					option.value = machine.id;
					option.textContent = machine.name;
					machineSelect.appendChild(option);
				});
			})
			.catch((error) => {
				console.error("Error loading machines:", error);
				showNotification("Failed to load machines", "error");
			});
	}
}

function populateFlagReasons() {
	const flagReasonSelect = document.getElementById("flag-reason");
	flagReasonSelect.innerHTML = '<option value="">Select Reason</option>';

	flagReasons.forEach((reason) => {
		const option = document.createElement("option");
		option.value = reason.id;
		option.textContent = reason.name;
		flagReasonSelect.appendChild(option);
	});
}

function setupEventListeners() {
	document.getElementById("machine").addEventListener("change", handleMachineChange);
	document.getElementById("shift-select").addEventListener("change", handleShiftDateChange);
	document.getElementById("date-select").addEventListener("change", handleShiftDateChange);
	document.getElementById("change-machine-btn").addEventListener("click", resetToMachineSelection);
	document.getElementById("input-form").addEventListener("submit", handleInputSubmit);
	document.getElementById("output-form").addEventListener("submit", handleOutputSubmit);
	document.getElementById("po-search").addEventListener("focus", handlePoSearchFocus);

	document.addEventListener("click", function (e) {
		const poSearchInput = document.getElementById("po-search");
		const poResultsDiv = document.getElementById("po-results");
		if (!poSearchInput.contains(e.target) && !poResultsDiv.contains(e.target)) {
			poResultsDiv.classList.add("hidden");
		}
	});

	const today = new Date().toISOString().split("T")[0];
	document.getElementById("date-select").value = today;
}

function handlePoSearchFocus() {
	const poSearchInput = document.getElementById("po-search");
	if (poSearchInput.disabled) return;
	if (Object.keys(processOrders).length > 0) {
		showAllPOs();
	}
}

async function handleShiftDateChange() {
	resetForm();
	await loadProcessOrders();
}

async function handleMachineChange(e) {
	const machineId = e.target.value;
	if (machineId) {
		document.getElementById("po-section").style.display = "block";
		resetForm();
		await loadProcessOrders();
	} else {
		document.getElementById("po-section").style.display = "none";
		resetForm();
	}
}

function resetForm() {
	document.getElementById("po-search").value = "";
	document.getElementById("po-search").disabled = true;
	document.getElementById("process-order").value = "";
	document.getElementById("po-results").innerHTML = "";
	document.getElementById("po-results").classList.add("hidden");
	document.getElementById("job-details").style.display = "none";
	document.getElementById("input-section").style.display = "none";
	document.getElementById("input-form").reset();
	document.querySelector("#input-form button[type='submit']").disabled = true;
	document.querySelector("#input-form button[type='submit']").textContent = "Start Production";
	document.getElementById("input-title").textContent = "Currently Consuming (Input)";
}

function displayJobDetails(poData) {
	const jobMaterials = document.getElementById("job-materials");
	if (!jobMaterials) return;

	jobMaterials.innerHTML = "";
	const materialDetails = poData.material_details || {};

	if (Object.keys(materialDetails).length === 0) {
		const emptyDiv = document.createElement("div");
		emptyDiv.className = "text-gray-500 text-center py-4";
		emptyDiv.textContent = "No material details";
		jobMaterials.appendChild(emptyDiv);
		return;
	}

	const grid = document.createElement("div");
	grid.className = "grid grid-cols-1 md:grid-cols-2 gap-3";

	Object.keys(materialDetails).forEach((key) => {
		const value = materialDetails[key];
		const card = document.createElement("div");
		card.className = "bg-white rounded-lg border border-gray-200 p-3 shadow-sm";

		const header = document.createElement("div");
		header.className = "font-semibold text-gray-800 mb-2 flex items-center gap-2";
		header.innerHTML = `
			<div class="w-8 h-8 rounded-md bg-blue-100 flex items-center justify-center">
				<i class="fas fa-box text-blue-600 text-sm"></i>
			</div>
			<span>${escapeHtml(key)}</span>
		`;

		const content = document.createElement("div");
		content.className = "text-sm text-gray-600";

		if (value.includes(",")) {
			const items = value.split(",").map((item) => item.trim());
			const list = document.createElement("div");
			list.className = "space-y-1";

			items.forEach((item) => {
				const itemDiv = document.createElement("div");
				itemDiv.className = "flex items-start gap-2 py-1";
				itemDiv.innerHTML = `
					<i class="fas fa-check-circle text-green-500 text-xs mt-1"></i>
					<span class="flex-1">${escapeHtml(item)}</span>
				`;
				list.appendChild(itemDiv);
			});
			content.appendChild(list);
		} else {
			const singleItem = document.createElement("div");
			singleItem.className = "flex items-start gap-2 py-1";
			singleItem.innerHTML = `
				<i class="fas fa-check-circle text-green-500 text-xs mt-1"></i>
				<span class="flex-1">${escapeHtml(value)}</span>
			`;
			content.appendChild(singleItem);
		}

		card.appendChild(header);
		card.appendChild(content);
		grid.appendChild(card);
	});

	jobMaterials.appendChild(grid);
}

function enableOutputForm() {
	document.getElementById("output-roll-no").value = generateOutputRollNo();
	document.querySelector("#output-form button[type='submit']").disabled = false;
	document.getElementById("output-section").style.display = "block";
	document.getElementById("active-production").style.display = "block";
}

function disableOutputForm() {
	document.getElementById("output-roll-no").value = "";
	document.querySelector("#output-form button[type='submit']").disabled = true;
	document.getElementById("output-section").style.display = "none";
	document.getElementById("active-production").style.display = "none";
}

function generateOutputRollNo() {
	const timestamp = new Date().getTime();
	const random = Math.floor(Math.random() * 1000);
	return `LP3-${timestamp.toString().slice(-3)}${random}`;
}

async function handleInputSubmit(e) {
	e.preventDefault();
	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const formData = {
		shift_id: parseInt(document.getElementById("shift-select").value),
		production_order: document.getElementById("process-order").value,
		batch_roll_no: document.getElementById("batch-roll-no").value,
		start_weight: parseFloat(document.getElementById("start-weight").value),
		start_meter: parseFloat(document.getElementById("start-meter").value),
		machine_id: parseInt(document.getElementById("machine").value),
	};

	try {
		const newJob = await createJob(formData);
		currentJob = newJob;

		showNotification("Job started successfully!", "success");

		enableOutputForm();

		submitBtn.disabled = true;
		submitBtn.textContent = "Production Started";

		document.getElementById("shift-select").disabled = true;
		document.getElementById("date-select").disabled = true;
		document.getElementById("machine").disabled = true;
		document.getElementById("po-search").disabled = true;
		document.getElementById("process-order").disabled = true;
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

async function handleOutputSubmit(e) {
	e.preventDefault();
	if (!currentJob) {
		showNotification("No active job selected", "error");
		return;
	}

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const formData = {
		output_roll_no: document.getElementById("output-roll-no").value,
		final_meter: parseFloat(document.getElementById("final-meter").value),
		number_of_flags: parseInt(document.getElementById("flag-count").value) || 0,
		flag_reason_id: document.getElementById("flag-reason").value ? parseInt(document.getElementById("flag-reason").value) : null,
		final_weight: 0,
		job_id: currentJob.id,
	};

	try {
		await createRoll(formData);
		showNotification("Output roll created successfully!", "success");
		e.target.reset();
		document.getElementById("output-roll-no").value = generateOutputRollNo();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

async function createJob(jobData) {
	const response = await fetch("/api/jobs/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(jobData),
	});
	return await handleApiResponse(response);
}

async function createRoll(rollData) {
	const response = await fetch("/api/rolls/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(rollData),
	});
	return await handleApiResponse(response);
}

function resetToMachineSelection() {
	document.getElementById("machine").value = "";
	document.getElementById("machine").disabled = false;
	document.getElementById("shift-select").value = "";
	document.getElementById("shift-select").disabled = false;
	document.getElementById("date-select").disabled = false;
	document.getElementById("po-section").style.display = "none";
	document.getElementById("output-section").style.display = "none";
	document.getElementById("active-production").style.display = "none";
	currentJob = null;
	disableOutputForm();
	resetForm();
}

function setSelectLoading(selectElement, loading) {
	if (loading) {
		selectElement.disabled = true;
		const originalHTML = selectElement.innerHTML;
		selectElement.setAttribute("data-original-html", originalHTML);
		selectElement.innerHTML = '<option value="">Loading...</option>';
	} else {
		selectElement.disabled = false;
		const originalHTML = selectElement.getAttribute("data-original-html");
		if (originalHTML && selectElement.children.length <= 1) {
			selectElement.innerHTML = originalHTML;
		}
	}
}
