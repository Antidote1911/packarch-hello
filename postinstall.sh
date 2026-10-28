#!/bin/sh

mkdir -p "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/packarch-hello/"
cp -r "${MESON_SOURCE_ROOT}/src/scripts" "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/packarch-hello/"
