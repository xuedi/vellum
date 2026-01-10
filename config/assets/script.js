/**
 * Vellum - Interactive Navigation
 * Handles section toggling, navigation buttons, and dropdown menus
 */

/**
 * Show a specific section and update navigation state
 * @param {HTMLElement} targetSection - The section to show
 * @param {HTMLElement} clickedBtn - The navigation button that was clicked (optional)
 */
function showSection(targetSection, clickedBtn) {
    // Remove active from all nav buttons
    document.querySelectorAll('#nav-buttons button').forEach(b => b.classList.remove('active'));
    clickedBtn?.classList.add('active');

    // Reset dropdown if showing a non-Related Documents section
    const dropdown = document.getElementById('related-docs-dropdown');
    if (dropdown && clickedBtn) {
        dropdown.selectedIndex = 0;
    }

    // Hide all sections
    document.querySelectorAll('#content > .section').forEach(s => {
        s.classList.add('hidden');
    });

    // Show and expand target section
    targetSection.classList.remove('hidden');
    targetSection.classList.remove('collapsed');

    // Scroll to top
    window.scrollTo({ top: 0, behavior: 'smooth' });
}

/**
 * Toggle a section's collapsed state
 * @param {HTMLElement} section - The section to toggle
 */
function toggleSection(section) {
    section.classList.toggle('collapsed');
}

/**
 * Show a specific subsection panel within Related Documents
 * @param {number} panelIndex - The index of the panel to show
 */
function showSubsectionPanel(panelIndex) {
    const relatedDocsSection = document.querySelector('.related-docs-section');
    if (!relatedDocsSection) return;

    // Hide all panels
    relatedDocsSection.querySelectorAll('.subsection-panel').forEach(p => p.classList.remove('active'));

    // Show selected panel
    const panels = relatedDocsSection.querySelectorAll('.subsection-panel');
    if (panels[panelIndex]) {
        panels[panelIndex].classList.add('active');
    }
}

/**
 * Convert Related Documents section into subsection panels controlled by dropdown
 * @param {HTMLElement} section - The section to convert
 * @param {HTMLElement} navContainer - The navigation container
 * @param {HTMLElement} originalButton - The original button to replace
 */
function createRelatedDocsDropdown(section, navContainer, originalButton) {
    const content = section.querySelector('.section-content');
    if (!content) return;

    // Mark section for identification
    section.classList.add('related-docs-section');

    // Hide the section header (navigation is via dropdown)
    const sectionHeader = section.querySelector('.section-header');
    if (sectionHeader) {
        sectionHeader.style.display = 'none';
    }

    // Find all H3 elements (subsections)
    const h3Elements = content.querySelectorAll('h3');
    if (h3Elements.length === 0) return;

    // Create dropdown
    const dropdown = document.createElement('select');
    dropdown.id = 'related-docs-dropdown';

    // Add disabled dropdown section name as label
    const labelOption = document.createElement('option');
    labelOption.textContent = DROPDOWN_SECTION || 'Select';
    labelOption.disabled = true;
    labelOption.selected = true;
    dropdown.appendChild(labelOption);

    // Create panels and dropdown options
    const panels = [];

    h3Elements.forEach((h3, index) => {
        // Create dropdown option
        const option = document.createElement('option');
        option.textContent = h3.textContent;
        option.value = index;
        dropdown.appendChild(option);

        // Create panel for content between this H3 and the next
        const panel = document.createElement('div');
        panel.className = 'subsection-panel';

        // Add the H3 title to the panel
        const title = h3.cloneNode(true);
        panel.appendChild(title);

        // Collect all siblings until next H3 or end
        let sibling = h3.nextElementSibling;
        const contentNodes = [];

        while (sibling && sibling.tagName !== 'H3') {
            contentNodes.push(sibling);
            sibling = sibling.nextElementSibling;
        }

        // Move content to panel
        contentNodes.forEach(node => panel.appendChild(node.cloneNode(true)));

        panels.push(panel);
    });

    // Set up dropdown change handler
    dropdown.onchange = function() {
        const selectedIndex = parseInt(this.value);

        // Remove active from all nav buttons
        document.querySelectorAll('#nav-buttons button').forEach(b => b.classList.remove('active'));

        // Show the Related Documents section
        showSection(section, null);

        // Show the selected subsection panel
        showSubsectionPanel(selectedIndex);

        // Close dropdown after selection
        this.blur();
    };

    // Replace button with dropdown
    originalButton.remove();
    navContainer.appendChild(dropdown);

    // Clear original content and add panels
    content.innerHTML = '';
    panels.forEach(panel => content.appendChild(panel));

    // Show first panel by default when section is shown
    if (panels.length > 0) {
        panels[0].classList.add('active');
    }
}

/**
 * Initialize navigation buttons and section visibility on page load
 */
document.addEventListener('DOMContentLoaded', function() {
    // Add click handlers to section headers
    document.querySelectorAll('.section-header').forEach(header => {
        header.onclick = () => toggleSection(header.parentElement);
    });

    // Find sections and buttons
    const sections = document.querySelectorAll('#content > .section');
    const navContainer = document.getElementById('nav-buttons');
    const buttons = Array.from(navContainer.querySelectorAll('button'));

    // Find "Related Documents" section
    let relatedDocsSection = null;
    let relatedDocsButton = null;
    let relatedDocsIndex = -1;

    buttons.forEach((btn, index) => {
        const sectionTitle = sections[index]?.querySelector('.section-header h2')?.textContent || '';

        if (DROPDOWN_SECTION && sectionTitle === DROPDOWN_SECTION) {
            relatedDocsSection = sections[index];
            relatedDocsButton = btn;
            relatedDocsIndex = index;
        } else {
            btn.onclick = () => showSection(sections[index], btn);
        }
    });

    // Convert configured section to dropdown
    if (DROPDOWN_SECTION && relatedDocsSection && relatedDocsButton) {
        createRelatedDocsDropdown(relatedDocsSection, navContainer, relatedDocsButton);
    }

    // Show first section on startup
    const remainingButtons = navContainer.querySelectorAll('button');
    if (sections.length > 0 && remainingButtons.length > 0) {
        showSection(sections[0], remainingButtons[0]);
    }
});
