// DSCode Documentation - Minimal Enhancements

// Add external link icons
document.addEventListener("DOMContentLoaded", function () {
  document.querySelectorAll("a[href^='http']").forEach(function (link) {
    if (!link.closest(".md-header, .md-footer, .md-source")) {
      link.setAttribute("target", "_blank");
      link.setAttribute("rel", "noopener noreferrer");
    }
  });
});
