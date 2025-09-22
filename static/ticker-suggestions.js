document.addEventListener("DOMContentLoaded", function () {
  // Find all suggestion items.
  const suggestionItems = document.querySelectorAll("suggestion-item");
  const tickerInput = document.getElementById("ticker_symbol");

  if (!tickerInput) {
    return; // No ticker input field on this page.
  }

  suggestionItems.forEach(function (item) {
    // Add click handler to each suggestion item.
    item.style.cursor = "pointer";
    item.addEventListener("click", function () {
      // Find the symbol within this suggestion item.
      const symbolElement = item.querySelector("suggestion-symbol");

      if (!symbolElement) {
        return;
      }

      const symbol = symbolElement.textContent.trim();

      tickerInput.value = symbol;
      tickerInput.focus();
    });
  });
});
