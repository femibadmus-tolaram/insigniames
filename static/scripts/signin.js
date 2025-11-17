/** @format */

const workflowSteps = [
	{
		url: "/static/scripts/downloads/lottiefiles/1.json",
	},
	{
		url: "/static/scripts/downloads/lottiefiles/2.json",
	},
	{
		url: "/static/scripts/downloads/lottiefiles/3.json",
	},
	{
		url: "/static/scripts/downloads/lottiefiles/4.json",
	},
];

function generateWorkflowSteps() {
	const container = document.getElementById("workflowContainer");

	workflowSteps.forEach((step, index) => {
		const stepElement = document.createElement("div");
		stepElement.className = `workflow-step ${index === 0 ? "active" : ""}`;
		stepElement.innerHTML = `
                        <div class="lottie-icon">
                            <lottie-player
                                src="${step.url}"
                                background="transparent"
                                speed="1"
                                style="width: 700px; height: 700px"
                                loop
                                autoplay
                            ></lottie-player>
                        </div>
                    `;
		container.appendChild(stepElement);
	});
}

let currentStep = 0;
let steps = [];

function animateWorkflow() {
	setTimeout(() => {
		steps[currentStep].classList.remove("active");
		steps[currentStep].classList.add("previous");

		currentStep = (currentStep + 1) % steps.length;

		steps.forEach((step) => {
			step.classList.remove("active", "previous");
		});

		steps[currentStep].classList.add("active");

		setTimeout(animateWorkflow, 3000);
	}, 3000);
}

document.addEventListener("DOMContentLoaded", function () {
	generateWorkflowSteps();

	steps = document.querySelectorAll(".workflow-step");

	setTimeout(animateWorkflow, 1000);

	const inputs = document.querySelectorAll("input");
	inputs.forEach((input) => {
		input.addEventListener("focus", function () {
			this.parentElement.style.transform = "scale(1.02)";
		});

		input.addEventListener("blur", function () {
			this.parentElement.style.transform = "scale(1)";
		});
	});
});

async function login() {
	const staffid = document.getElementById("staffid").value.trimEnd();
	const password = document.getElementById("password").value;
	const errorMsg = document.getElementById("error-msg");
	const loginBtn = document.getElementById("loginBtn");

	loginBtn.disabled = true;
	loginBtn.innerHTML = "Logging in...";

	try {
		const res = await fetch("", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ staffid, password }),
		});

		if (res.ok) {
			const user = await res.json();
			sessionStorage.setItem("user", JSON.stringify(user));
			window.location.href = window.location.origin + "/";
		} else {
			loginBtn.disabled = false;
			const text = await res.text();
			errorMsg.innerText = text;
			errorMsg.classList.add("visible");
		}
	} catch (err) {
		loginBtn.disabled = false;
		errorMsg.innerText = err.message;
		errorMsg.classList.add("visible");
	} finally {
		loginBtn.innerHTML = "Log In";
		setTimeout(() => {
			errorMsg.classList.remove("visible");
		}, 5000);
	}
}
