// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="intro.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="before.html">Before using</a></li><li class="chapter-item expanded affix "><a href="glossary.html">Glossary</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="setup.html"><strong aria-hidden="true">1.</strong> Project Setup</a></li><li class="chapter-item expanded "><a href="inject.html"><strong aria-hidden="true">2.</strong> Injecting Objects</a></li><li class="chapter-item expanded "><a href="request.html"><strong aria-hidden="true">3.</strong> Requesting Objects</a></li><li class="chapter-item expanded "><a href="provides.html"><strong aria-hidden="true">4.</strong> Providing Objects</a></li><li class="chapter-item expanded "><a href="builder.html"><strong aria-hidden="true">5.</strong> Builder Modules</a></li><li class="chapter-item expanded "><a href="binds.html"><strong aria-hidden="true">6.</strong> Binding traits</a></li><li class="chapter-item expanded "><a href="scoped.html"><strong aria-hidden="true">7.</strong> Scoped Bindings</a></li><li class="chapter-item expanded "><a href="qualifiers.html"><strong aria-hidden="true">8.</strong> Qualifiers</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="provider.html"><strong aria-hidden="true">9.</strong> Provider</a></li><li class="chapter-item expanded "><a href="lazy.html"><strong aria-hidden="true">10.</strong> Lazy</a></li><li class="chapter-item expanded "><a href="factory.html"><strong aria-hidden="true">11.</strong> Factory</a></li><li class="chapter-item expanded "><a href="optional.html"><strong aria-hidden="true">12.</strong> Optional Bindings</a></li><li class="chapter-item expanded "><a href="multibindings.html"><strong aria-hidden="true">13.</strong> Multibindings</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="vec.html"><strong aria-hidden="true">13.1.</strong> Vec bindings</a></li><li class="chapter-item expanded "><a href="map.html"><strong aria-hidden="true">13.2.</strong> Map bindings</a></li><li class="chapter-item expanded "><a href="empty_multibinding.html"><strong aria-hidden="true">13.3.</strong> Empty multibindings</a></li></ol></li><li class="chapter-item expanded "><a href="subcomponent.html"><strong aria-hidden="true">14.</strong> Subcomponents</a></li><li class="chapter-item expanded "><a href="define_component.html"><strong aria-hidden="true">15.</strong> Defined components</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="caveats.html"><strong aria-hidden="true">16.</strong> Caveats</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path_resolution.html"><strong aria-hidden="true">16.1.</strong> Path resolution</a></li><li class="chapter-item expanded "><a href="cross_macro_communication.html"><strong aria-hidden="true">16.2.</strong> Cross macro communication</a></li><li class="chapter-item expanded "><a href="visibility_bypass.html"><strong aria-hidden="true">16.3.</strong> Bypassing visibility</a></li><li class="chapter-item expanded "><a href="late_impl_generation.html"><strong aria-hidden="true">16.4.</strong> Late implementation generation</a></li></ol></li><li class="chapter-item expanded "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="code-of-conduct.html">Code of conduct</a></li><li class="chapter-item expanded affix "><a href="contributing.html">Contributing</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
