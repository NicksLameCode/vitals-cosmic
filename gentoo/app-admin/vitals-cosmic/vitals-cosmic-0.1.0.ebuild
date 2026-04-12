# Copyright 2026 Nick
# Distributed under the terms of the BSD 3-Clause License

EAPI=8

CRATES=""

inherit cargo desktop xdg

DESCRIPTION="COSMIC applet for system vitals (companion to vitals-rs)"
HOMEPAGE="https://github.com/NicksLameCode/vitals-cosmic"

# For a personal overlay you'll usually use a local tarball or git snapshot;
# fill this in when you cut a release.
SRC_URI="
	https://github.com/NicksLameCode/vitals-cosmic/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz
	${CARGO_CRATE_URIS}
"

LICENSE="BSD"
SLOT="0"
KEYWORDS="~amd64"

# vitals-cosmic is a thin D-Bus client of com.corecoding.Vitals, so the
# daemon from app-admin/vitals-rs must be installed for this applet to
# actually show any readings. cosmic-base provides the COSMIC panel.
RDEPEND="
	>=app-admin/vitals-rs-1.0.0
	cosmic-base/cosmic-base
	dev-libs/wayland
"
DEPEND="${RDEPEND}"
BDEPEND="
	virtual/rust
	dev-util/pkgconf
"

QA_FLAGS_IGNORED="usr/bin/${PN}"

src_install() {
	cargo_src_install

	domenu data/com.corecoding.VitalsCosmic.desktop

	insinto /usr/share/icons/hicolor/scalable/apps
	doins data/icons/*.svg
}

pkg_postinst() {
	xdg_desktop_database_update
	xdg_icon_cache_update

	elog "vitals-cosmic needs com.corecoding.Vitals to be running on the session"
	elog "bus. If the vitals-rs package doesn't auto-activate it, you can run"
	elog "vitals-daemon manually, or add it to your cosmic-session autostart."
	elog ""
	elog "Enable the applet from cosmic-settings → Desktop → Panel → Applets."
}

pkg_postrm() {
	xdg_desktop_database_update
	xdg_icon_cache_update
}
