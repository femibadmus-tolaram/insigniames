/** @format */

let currentUser = null;
let machines = [];
let processOrders = [];
let flagReasons = [];
let currentJob = null;
let activeJobs = [];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadCurrentUser();
	await loadMachines();
	await loadFlagReasons();
	setupEventListeners();
	autoSelectShift();
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
	const processOrderSelect = document.getElementById("process-order");
	setSelectLoading(processOrderSelect, true);

	try {
		const response = await fetch("/api/sap/process_order");
		processOrders = await handleApiResponse(response);
		console.log(processOrders);

		processOrderSelect.innerHTML = '<option value="">Select Process Order</option>';
		processOrders.forEach((po, index) => {
			console.log(`Creating option ${index}:`, po.ManufacturingOrder);
			const option = document.createElement("option");
			option.value = po.ManufacturingOrder;
			option.textContent = `${po.ManufacturingOrder} - ${po.TotalQuantity} ${po.Unit}`;
			option.setAttribute("data-po", JSON.stringify(po));
			console.log("Option created:", option);
			processOrderSelect.appendChild(option);
		});
		// processOrders.forEach((po) => {
		// 	const option = document.createElement("option");
		// 	option.value = po.ManufacturingOrder;
		// 	option.textContent = `${po.ManufacturingOrder} - ${po.TotalQuantity} ${po.Unit}`;
		// 	option.setAttribute("data-po", JSON.stringify(po));
		// 	processOrderSelect.appendChild(option);
		// });
	} catch (error) {
		showNotification(error.message, "error");
		processOrderSelect.innerHTML = '<option value="">Failed to load POs</option>';
	} finally {
		setSelectLoading(processOrderSelect, false);
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

	if (currentUser && currentUser.machine_ids) {
		const userMachines = machines.filter((machine) => currentUser.machine_ids.includes(machine.id));

		userMachines.forEach((machine) => {
			const option = document.createElement("option");
			option.value = machine.id;
			option.textContent = machine.name;
			machineSelect.appendChild(option);
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
	document.getElementById("process-order").addEventListener("change", handleProcessOrderChange);
	document.getElementById("change-machine-btn").addEventListener("click", resetToMachineSelection);
	document.getElementById("input-form").addEventListener("submit", handleInputSubmit);
	document.getElementById("output-form").addEventListener("submit", handleOutputSubmit);
	document.getElementById("refresh-btn").addEventListener("click", refreshProductionData);
	document.getElementById("active-job").addEventListener("change", handleActiveJobChange);
}

function handleMachineChange(e) {
	const machineId = e.target.value;
	if (machineId) {
		document.getElementById("po-section").style.display = "block";
		loadProcessOrders();
		loadActiveJobs(machineId);
	} else {
		document.getElementById("po-section").style.display = "none";
		document.getElementById("production-forms").style.display = "none";
		document.getElementById("active-production").style.display = "none";
	}
}

function handleProcessOrderChange(e) {
	const selectedOption = e.target.options[e.target.selectedIndex];
	const jobDetails = document.getElementById("job-details");

	if (selectedOption.value) {
		const poData = JSON.parse(selectedOption.getAttribute("data-po"));
		displayJobDetails(poData);
		jobDetails.style.display = "block";
		document.getElementById("production-forms").style.display = "grid";
	} else {
		jobDetails.style.display = "none";
		document.getElementById("production-forms").style.display = "none";
	}
}

function displayJobDetails(poData) {
	document.getElementById("job-sku").textContent = poData.ManufacturingOrder || "-";
	document.getElementById("job-film").textContent = "20mic 1840mm Bopp"; // This would come from PO data
	document.getElementById("job-materials").textContent = extractMaterials(poData);
}

function extractMaterials(poData) {
	if (poData.Components && poData.Components.length > 0) {
		return poData.Components.map((comp) => `${comp.Material} (${comp.RequiredQuantity})`).join(", ");
	}
	return "No materials specified";
}

async function loadActiveJobs(machineId) {
	try {
		const response = await fetch(`/api/jobs/filter?machine_id=${machineId}&per_page=100`);
		const result = await handleApiResponse(response);
		activeJobs = result.data || result;

		const activeJobSelect = document.getElementById("active-job");
		activeJobSelect.innerHTML = '<option value="">No active job</option>';

		const activeJob = activeJobs.find((job) => !job.completed_at);
		if (activeJob) {
			currentJob = activeJob;
			activeJobSelect.innerHTML = `<option value="${activeJob.id}">Job #${activeJob.id} - ${activeJob.batch_roll_no}</option>`;
			activeJobSelect.disabled = false;
			enableOutputForm();
			loadJobRolls(activeJob.id);
		} else {
			currentJob = null;
			activeJobSelect.disabled = true;
			disableOutputForm();
		}
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function handleActiveJobChange(e) {
	const jobId = e.target.value;
	if (jobId) {
		currentJob = activeJobs.find((job) => job.id == jobId);
		enableOutputForm();
		loadJobRolls(jobId);
	} else {
		currentJob = null;
		disableOutputForm();
	}
}

function enableOutputForm() {
	document.getElementById("output-roll-no").value = generateOutputRollNo();
	document.querySelector("#output-form button[type='submit']").disabled = false;
	document.getElementById("active-production").style.display = "block";
}

function disableOutputForm() {
	document.getElementById("output-roll-no").value = "";
	document.querySelector("#output-form button[type='submit']").disabled = true;
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
		shift_id: parseInt(document.getElementById("shift").value),
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
		e.target.reset();
		autoSelectShift();

		// Update active job dropdown
		const activeJobSelect = document.getElementById("active-job");
		activeJobSelect.innerHTML = `<option value="${newJob.id}">Job #${newJob.id} - ${newJob.batch_roll_no}</option>`;
		activeJobSelect.disabled = false;

		enableOutputForm();
		loadJobRolls(newJob.id);

		// Disable PO selection and input form
		document.getElementById("process-order").disabled = true;
		document
			.getElementById("input-form")
			.querySelectorAll("input, select, button")
			.forEach((el) => {
				if (el.type !== "submit") el.disabled = true;
			});
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
		final_weight: parseFloat(document.getElementById("final-weight").value),
		job_id: currentJob.id,
	};

	try {
		await createRoll(formData);
		showNotification("Output roll created successfully!", "success");
		e.target.reset();
		document.getElementById("output-roll-no").value = generateOutputRollNo();
		loadJobRolls(currentJob.id);
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

async function loadJobRolls(jobId) {
	try {
		const response = await fetch(`/api/rolls/filter?job_id=${jobId}&per_page=10`);
		const result = await handleApiResponse(response);
		const rolls = result.data || result;
		renderRollsTable(rolls);
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function renderRollsTable(rolls) {
	const tbody = document.getElementById("rolls-table-body");

	if (!rolls || rolls.length === 0) {
		tbody.innerHTML = '<tr><td colspan="7" class="text-center text-gray-500 py-4">No rolls found</td></tr>';
		return;
	}

	tbody.innerHTML = rolls
		.map(
			(roll) => `
        <tr class="hover:bg-gray-50">
            <td class="py-3 px-4 font-medium">${escapeHtml(roll.output_roll_no)}</td>
            <td class="py-3 px-4">
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
					roll.number_of_flags > 0 ? "bg-red-100 text-red-800" : "bg-green-100 text-green-800"
				}">
                    ${roll.number_of_flags > 0 ? "Flagged" : "Completed"}
                </span>
            </td>
            <td class="py-3 px-4 text-center">${(roll.final_weight || 0).toFixed(2)}</td>
            <td class="py-3 px-4 text-center">${(roll.final_meter || 0).toFixed(2)}</td>
            <td class="py-3 px-4">${escapeHtml(currentJob?.batch_roll_no || "-")}</td>
            <td class="py-3 px-4">${escapeHtml(flagReasons.find((r) => r.id === roll.flag_reason_id)?.name || "-")}</td>
            <td class="py-3 px-4 text-center">${roll.number_of_flags || 0}</td>
        </tr>
    `
		)
		.join("");
}

function resetToMachineSelection() {
	document.getElementById("machine").value = "";
	document.getElementById("po-section").style.display = "none";
	document.getElementById("production-forms").style.display = "none";
	document.getElementById("active-production").style.display = "none";
	document.getElementById("job-details").style.display = "none";

	// Re-enable PO selection and input form
	document.getElementById("process-order").disabled = false;
	document
		.getElementById("input-form")
		.querySelectorAll("input, select, button")
		.forEach((el) => {
			el.disabled = false;
		});

	currentJob = null;
}

function refreshProductionData() {
	const machineId = document.getElementById("machine").value;
	if (machineId) {
		loadActiveJobs(machineId);
		if (currentJob) {
			loadJobRolls(currentJob.id);
		}
	}
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

function autoSelectShift() {
	const now = new Date();
	const hours = now.getHours();
	const shiftValue = hours >= 7 && hours < 19 ? "1" : "2";
	document.getElementById("shift").value = shiftValue;
}
