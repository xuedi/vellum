/**
 * Vellum - Minimal Panel Navigation
 * Simple show/hide logic for pre-rendered panels
 */

document.addEventListener('DOMContentLoaded', function() {
    const buttons = document.querySelectorAll('#nav-buttons button');
    const dropdown = document.getElementById('dropdown');
    const panels = document.querySelectorAll('.panel');

    function showPanel(panelId) {
        // Hide all panels
        panels.forEach(p => p.classList.add('hidden'));

        // Show selected panel
        const panel = document.getElementById('panel-' + panelId);
        if (panel) {
            panel.classList.remove('hidden');
        }

        // Reset button active states
        buttons.forEach(b => b.classList.remove('active'));

        // Scroll to top
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    // Set up button click handlers
    buttons.forEach(btn => {
        btn.onclick = () => {
            showPanel(btn.dataset.panel);
            btn.classList.add('active');
            if (dropdown) dropdown.selectedIndex = 0;
        };
    });

    // Set up dropdown change handler
    if (dropdown) {
        dropdown.onchange = function() {
            const option = this.options[this.selectedIndex];
            if (option && option.dataset.panel) {
                showPanel(option.dataset.panel);
            }
        };
    }

    // Show first panel on load
    if (buttons.length > 0) {
        buttons[0].click();
    }
});
