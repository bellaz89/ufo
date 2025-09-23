import align
import aperture
import chroma
import closed_orbit
import elements
import optics2
import optics3
import radiation
import thick_vs_kick
import tracking

import optics

print("\n", 80 * "#", "\n", 35 * " ", "LOAD ELEMENTS + PASS METHOD\n", 80 * "#", "\n")
elements.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "OPTICS-1 (PERIODIC)\n" + 80 * "#" + "\n")
optics.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "OPTICS-2 (PROPAGATE)\n" + 80 * "#" + "\n")
optics2.test()

print(
    "\n"
    + 80 * "#"
    + "\n"
    + 25 * " "
    + "OPTICS-3 (CHROMATICITY FROM TRACKING)\n"
    + 80 * "#"
    + "\n"
)
optics3.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "THICK Vs KICK\n" + 80 * "#" + "\n")
thick_vs_kick.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "CHROMATICITY\n" + 80 * "#" + "\n")
chroma.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "RADIATION\n" + 80 * "#" + "\n")
radiation.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "TRACKING\n" + 80 * "#" + "\n")
tracking.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "ALIGN\n" + 80 * "#" + "\n")
align.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "CLOSED ORBIT\n" + 80 * "#" + "\n")
closed_orbit.test()

print("\n" + 80 * "#" + "\n" + 35 * " " + "APERTURE\n" + 80 * "#" + "\n")
aperture.test()
