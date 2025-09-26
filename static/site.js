/**
 * Check for horizontal overflow in the navigation bar on page load and tag
 * the nav element if overflow is detected. This is used then on CSS to display
 * a scroll indicator.
 */
document.addEventListener("DOMContentLoaded", function () {
  const nav = document.querySelector("nav");
  const navUl = document.querySelector("nav > ul");

  if (nav && navUl) {
    function updateScrollIndicator() {
      const hasOverflow = navUl.scrollWidth > navUl.clientWidth;
      const isAtEnd =
        navUl.scrollLeft >= navUl.scrollWidth - navUl.clientWidth - 5; // 5px tolerance.

      if (hasOverflow && !isAtEnd) {
        nav.classList.add("has-overflow");
      } else {
        nav.classList.remove("has-overflow");
      }
    }

    updateScrollIndicator();
    window.addEventListener("resize", updateScrollIndicator);
    navUl.addEventListener("scroll", updateScrollIndicator);
  }
});
