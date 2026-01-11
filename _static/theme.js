window.addEventListener("load", (_) => {
  "use strict";

  /** COMPONENT X **/
  (function () {
    // TODO
  })();

  /** Dark/Light switch **/
  (function () {
    function prefersDark() {
      // if localStorage is set, use that
      const storedMode = localStorage.getItem("docdustry-dark-mode");
      if (storedMode === "dark") return true;
      if (storedMode === "light") return false;
      // if not, use browser default
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    }

    // Add button to toggle dark/light to docustry-switches tag
    const toggleButton = document.createElement("button");
    toggleButton.setAttribute("alt", "Toggle dark/light mode");
    toggleButton.addEventListener("click", () => {
      if (prefersDark()) {
        setLight();
      } else {
        setDark();
      }
    });
    const switches = document.querySelector("docdustry-switches");
    switches.appendChild(toggleButton);

    const body = document.body;
    function setDark(dark) {
      body.classList.add("dark-mode");
      localStorage.setItem("docdustry-dark-mode", "dark");
      toggleButton.textContent = "🌔";
    }
    function setLight(light) {
      body.classList.remove("dark-mode");
      localStorage.setItem("docdustry-dark-mode", "light");
      toggleButton.textContent = "🌘";
    }
    // Set body class accordingly
    if (prefersDark()) {
      setDark();
    } else {
      setLight();
    }
  })();

  console.log("Hello world!");
});
