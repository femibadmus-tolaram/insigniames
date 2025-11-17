/** @format */

document.addEventListener("DOMContentLoaded", function () {
	loadSettings();
	loadLastUpdate();
	setupEventListeners();
});

async function loadSettings() {
	try {
		const response = await fetch("http://localhost:8080/api/settings");
		if (!response.ok) throw new Error("Failed to load settings");

		const config = await response.json();
		populateForm(config);
		checkConfigStatus(config);
	} catch (error) {
		document.getElementById("update-app").style.display = "None";
		// showNotification("Error loading settings: " + error.message, "error");
		showNotification("App is not running", "error");
	}
}

async function loadLastUpdate() {
	try {
		const response = await fetch("http://localhost:8080/api/app/last_update");
		if (response.ok) {
			const lastUpdate = await response.text();
			document.getElementById("last-update").textContent = `Last update: ${lastUpdate}`;
		}
	} catch (error) {
		document.getElementById("last-update").textContent = "Last update: Unknown";
	}
}

async function updateApp() {
	const btn = document.getElementById("update-app");
	setButtonLoading(btn, true);

	try {
		const response = await fetch("http://localhost:8080/api/app/update");
		const result = await response.text();

		if (response.ok) {
			await new Promise((r) => setTimeout(r, 10000));
			loadLastUpdate();
			showNotification("Application updated successfully", "success");
		} else {
			showNotification("something went wrong, Try again later!", "error");
		}
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(btn, false);
	}
}

function populateForm(config) {
	document.getElementById("scanner-port").value = config.scanner.port_name || "";
	document.getElementById("scanner-baud").value = config.scanner.baud_rate || 9600;
	document.getElementById("scale-port").value = config.scale.port_name || "";
	document.getElementById("scale-baud").value = config.scale.baud_rate || 9600;
}

function setupEventListeners() {
	document.getElementById("update-app").addEventListener("click", updateApp);
	document.getElementById("settings-form").addEventListener("submit", handleSaveSettings);
	document.getElementById("test-scanner").addEventListener("click", testScannerConnection);
	document.getElementById("test-scale").addEventListener("click", testScaleConnection);
	document.getElementById("test-printer").addEventListener("click", testPrinterConnection);
}

async function handleSaveSettings(e) {
	e.preventDefault();
	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const config = {
		scanner: {
			port_name: document.getElementById("scanner-port").value,
			baud_rate: parseInt(document.getElementById("scanner-baud").value),
		},
		scale: {
			port_name: document.getElementById("scale-port").value,
			baud_rate: parseInt(document.getElementById("scale-baud").value),
		},
	};

	try {
		const response = await fetch("http://localhost:8080/api/settings/update", {
			method: "PUT",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(config),
		});

		let errorText;
		try {
			errorText = await response.json();
			errorText = errorText.message;
		} catch {
			errorText = await response.text();
		}

		showNotification("Settings saved successfully!", "success");
		checkConfigStatus(config);
	} catch (error) {
		showNotification(error.message || "Error saving settings", "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

async function testScannerConnection() {
	const btn = document.getElementById("test-scanner");
	setButtonLoading(btn, true);

	try {
		const response = await fetch("http://localhost:8080/api/settings/test/scanner");
		const result = await response.json();
		showNotification(result.message, result.success ? "success" : "error");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(btn, false);
	}
}

async function testScaleConnection() {
	const btn = document.getElementById("test-scale");
	setButtonLoading(btn, true);

	try {
		const response = await fetch("http://localhost:8080/api/settings/test/scale");
		const result = await response.json();
		showNotification(result.message, result.success ? "success" : "error");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(btn, false);
	}
}

async function testPrinterConnection() {
	const btn = document.getElementById("test-printer");
	setButtonLoading(btn, true);

	try {
		const response = await fetch("http://localhost:8080/api/settings/test/printer");
		const result = await response.json();
		showNotification(result.message, result.success ? "success" : "error");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(btn, false);
	}
}

function setButtonLoading(button, loading) {
	if (loading) {
		if (!button.getAttribute("data-original-text")) {
			button.setAttribute("data-original-text", button.innerHTML);
		}
		button.disabled = true;
		button.classList.add("btn-loading");
		button.innerHTML = '<i class="fas fa-spinner"></i> Loading...';
	} else {
		button.disabled = false;
		button.classList.remove("btn-loading");
		const originalText = button.getAttribute("data-original-text");
		button.innerHTML = originalText;
	}
}

function checkConfigStatus(config) {
	const statusDiv = document.getElementById("config-status");
	const message = document.getElementById("status-message");

	if (config.is_config_complete && config.is_config_complete()) {
		statusDiv.className = "bg-green-50 p-4 rounded-lg border border-green-200";
		message.textContent = "Configuration is complete and ready to use.";
		statusDiv.classList.remove("hidden");
	} else {
		statusDiv.className = "bg-yellow-50 p-4 rounded-lg border border-yellow-200";
		message.textContent = "Configuration is incomplete. Please check required fields.";
		statusDiv.classList.remove("hidden");
	}
}
