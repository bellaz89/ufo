
#define FLAG_LINEAR           (0x1 << 0)
#define FLAG_FIVED            (0x1 << 1)
#define FLAG_EXACT            (0x1 << 2)
#define FLAG_KICK             (0x1 << 3)
#define FLAG_RADIATION        (0x1 << 4)
#define FLAG_DOUBLE_PRECISION (0x1 << 5)
#define FLAG_ACHROMATIC       (0x1 << 6)

inline void align(particle_work_t* part_data,
                  const flags_t flags, 
                  const intarg_t args0, 
                  const intarg_t args1,
                  __local const float_t* args0_arr, 
                  __local const float_t* args1_arr){

  part_data->particle.x -= args0_arr[0];
  part_data->particle.y -= args0_arr[1];
}

inline void drift(particle_work_t* part_data,
                  const flags_t flags, 
                  const intarg_t args0, 
                  const intarg_t args1,
                  __local const float_t* args0_arr, 
                  __local const float_t* args1_arr){

  if (flags & FLAG_EXACT) {

  } else {
    part_data->particle.x += part_data->particle.px *  args0_arr[0];
    part_data->particle.y += part_data->particle.py *  args0_arr[0];
  }
  
  
}


//inline void drift(flags, length):
//    if flags & EXACT:
//        fragment  = ('{\n'
//                     '    ufloat opdp, l;\n'
//
//                     '    opdp = 1. + dp;\n'
//                    f'    l = {length} / sqrt(opdp*opdp - px*px - py*py);\n'
//                     '    x += px * l;\n'
//                     '    y += py * l;\n'
//                     '}\n')
// 
//    else:
//        fragment = (f'x += px * {(length)};\n'
//                    f'y += py * {(length)};\n')
//
//    return fragment
//
//inline void kick(flags, knl, ksl):
//    fragment = ''
//
//    if knl:
//        n = len(knl) #Avoid non-linear terms when LINEAR
//        max_order = min(n, 2) if (flags & LINEAR) else n 
//
//        dpx = f'{knl[max_order - 1]} {"" if flags & ACHROMATIC else "* oodppo"}'
//        dpy =  '0.'
//
//        for order in range(max_order - 1, 0, -1):
//            aux = f'({dpx} * x - ({dpy} * y)) / {order}'
//            dpy = f'({dpx} * y + ({dpy} * x)) / {order}'
//            dpx = f'({knl[order - 1]} {"" if flags & ACHROMATIC else "* oodppo"} + {aux})'
//
//        fragment = f'px -= {dpx};\npy += {dpy};\n\n'
//
//    if ksl:
//        n = len(ksl) #Avoid non-linear terms when LINEAR
//        max_order = min(n, 2) if (flags & LINEAR) else n 
//
//        dpy = f'{ksl[max_order - 1]} {"" if flags & ACHROMATIC else "* oodppo"}'
//        dpx =  '0.'
//
//        for order in range(max_order - 1, 0, -1):
//            aux = f'({dpx} * y + ({dpy} * x)) / {order}'
//            dpx = f'({dpx} * x - ({dpy} * y)) / {order}'
//            dpy = f'({ksl[order - 1]} {"" if flags & ACHROMATIC else "* oodppo"} + {aux})'
//
//        fragment += f'px -= {dpx};\npy += {dpy};\n\n'
//
//    return fragment
//
//#https://accelconf.web.cern.ch/IPAC2013/papers/mopwo027.pdf
//inline void teapot(flags, length, slices, knl, ksl, angle=0): #Using Teapot slicing
//    outer = f'({length} / 2.)' if slices == 1 else f'({length} / (2 * (1 + {slices})))' #Outer drifts length
//    inner = f'({length} - 2. * {outer}) / {slices - 1.}' #Inner drifts length
//
//    #Weak focusing
//    weak = f'px -= x * (({angle}) * ({angle}) * ({length})/{slices} / ({length}) / ({length}));\n'
//
//    knl = [f'({k}*{length}/{slices})' for k in knl]
//    ksl = [f'({k}*{length}/{slices})' for k in ksl]
//
//    fragment = drift(flags, outer)
//    for count in range(slices - 1):
//        fragment += kick(flags, knl, ksl) + weak
//        fragment += drift(flags, inner)
//
//    fragment += kick(flags, knl, ksl) + weak
//    fragment += drift(flags, outer)
//
//    return fragment
//
//inline void quadrupole(flags, slices, length, k1, dkn, dks):
//    if flags & KICK:
//        knl = copy_multipoles_and_pad(dkn, 2)
//        knl[1] = f'({knl[1]} + {k1})'
//
//        fragment = teapot(flags, length, slices, knl, dks)
//
//    else:
//        fragment = ('{\n'
//                    '    ufloat x0  = x;\n'
//                    '    ufloat px0 = px;\n'
//                    '    ufloat y0  = y;\n'
//                    '    ufloat py0 = py;\n'
//
//                   f'    ufloat k         = {k1};\n'
//                   f'    ufloat k2        = sqrt(fabs(k {"" if flags & ACHROMATIC else "* oodppo"}));\n'
//                   f'    ufloat k2l       = {length} * k2;\n'
//                    '    ufloat C         = cos( k2l);\n'
//                    '    ufloat CH        = cosh(k2l);\n'
//                    '    ufloat S         = sin( k2l);\n'
//                    '    ufloat SH        = sinh(k2l);\n'
//
//                    '    if (k > 0.) {\n'
//                    '        x  =  C  * x0      + S  * px0 / k2;\n'
//                    '        px = -S  * x0 * k2 + C  * px0;\n'
//                    '        y  =  CH * y0      + SH * py0 / k2;\n'
//                    '        py =  SH * y0 * k2 + CH * py0;\n'
//                    '    } else {\n'
//                    '        x  =  CH * x0      + SH * px0 / k2;\n'
//                    '        px =  SH * x0 * k2 + CH * px0;\n'
//                    '        y  =  C  * y0      + S  * py0 / k2;\n'
//                    '        py = -S  * y0 * k2 + C  * py0;\n'
//                    '   }\n'
//                    '}\n')
//
//    return fragment
//
//inline void sbend(flags, slices, length, angle, k1, dkn, dks):
//    if flags & KICK:
//        knl = copy_multipoles_and_pad(dkn, 2)
//        knl[1] = f'({knl[1]} + {k1})'
//        fragment = teapot(flags, length, slices, knl, dks, angle)
//    else:
//        fragment = ('{\n'
//                    '    ufloat x0  = x;\n'
//                    '    ufloat px0 = px;\n'
//                    '    ufloat y0  = y;\n'
//                    '    ufloat py0 = py;\n'
//
//                   f'    ufloat curvature = {angle} / ({length}) {"" if flags & ACHROMATIC else "* oodppo"};\n'
//                   f'    ufloat k         = {k1} {"" if flags & ACHROMATIC else "* oodppo"};\n'
//                    '    ufloat k2        = sqrt(fabs(k));\n'
//                   f'    ufloat k2l       = {length} * k2;\n'
//                    '    ufloat C         = cos( k2l);\n'
//                    '    ufloat CH        = cosh(k2l);\n'
//                    '    ufloat S         = sin( k2l);\n'
//                    '    ufloat SH        = sinh(k2l);\n'
//
//                    '    if (k > 0.) {\n'
//                    '        y  =  CH * y0      + SH * py0 / k2;\n'
//                    '        py =  SH * y0 * k2 + CH * py0;\n'
//                    '    }\n'
//
//                    '    if (k < 0.) {\n'
//                    '        y  =  C  * y0      + S  * py0 / k2;\n'
//                    '        py = -S  * y0 * k2 + C  * py0;\n'
//                    '    }\n'
//
//                    '    if (k == 0.)\n'
//                   f'        y += {length} * py0;\n'
//
//                   f'    k += curvature * curvature {"" if flags & ACHROMATIC else "/ oodppo"};\n' #Horizontal plane includes weak focusing 
//                    '    k2  = sqrt(fabs(k));\n'
//                   f'    k2l = {length} * k2;\n'
//                    '    C   = cos( k2l);\n'
//                    '    CH  = cosh(k2l);\n'
//                    '    S   = sin( k2l);\n'
//                    '    SH  = sinh(k2l);\n'
//
//                    '    if (k > 0.) {\n'
//                    '        x   =  C  * x0      + S  * px0 / k2;\n'
//                   f'        x  += dp * curvature * (1. - C) / fabs(k);\n'
//                    '        px  = -S  * x0 * k2 + C  * px0;\n'
//                   f'        px += dp * curvature * S / k2;\n'
//                    '    }\n'
//
//                    '    if (k < 0.) {\n'
//                    '        x  =  CH * x0      + SH * px0 / k2;\n'
//                   f'        x +=  dp * curvature * (CH - 1.) / fabs(k);\n'
//                    '        px =  SH * x0 * k2 + CH * px0;\n'
//                   f'        px += dp * curvature * SH / k2;\n'
//                    '    }\n'
//
//                    '    if (k == 0.)\n'
//                   f'        x += {length} * px0;\n'
//                    '}\n')
//
//    return fragment
//
//#For fringe field see SLAC-75
//inline void edge(flags, length, angle, e, fringe):
//    fragment = ('{\n'
//               f'    ufloat curvature, psi;\n'
//
//               f'    curvature = {angle} / ({length}) {"" if flags & ACHROMATIC else "* oodppo"};\n'
//               f'    psi = {e} - 2 * curvature * {fringe} / cos({e}) * (1. + sin({e}) * sin({e}));\n'
//
//               f'    px += x * curvature * tan({e});\n'
//               #f'    double psi = {e} - 2 * curvature * {fringe};\n' #First order
//               f'    py -= y * curvature * tan(psi);\n'
//                '}\n')
//
//    return fragment
//
//inline void sextupole(flags, slices, length, k2, k2s, dkn, dks):
//    knl = copy_multipoles_and_pad(dkn, 3)
//    ksl = copy_multipoles_and_pad(dks, 3)
//    knl[2] = f'({knl[2]} + {k2})'
//    ksl[2] = f'({ksl[2]} + {k2s})'
//
//    return teapot(flags, length, slices, knl, dks)
//
//inline void octupole(flags, slices, length, k3, k3s, dkn, dks):
//    knl = copy_multipoles_and_pad(dkn, 4)
//    ksl = copy_multipoles_and_pad(dks, 4)
//    knl[3] = f'({knl[3]} + {k3})'
//    ksl[3] = f'({ksl[3]} + {k3s})'
//
//    return teapot(flags, length, slices, knl, dks)
//
//#A wire parallel to the beam, with coordinates x, y
//inline void wire(x, y, k):
//    fragment = ('{\n'
//                '    ufloat alpha, B;\n'
//
//               f'     alpha = atan2({y} - y, {x} - x);\n'
//               f'     B = ({k}) / hypot({x} - x, {y} - y);\n'
//
//                '     px += B * cos(alpha);\n'
//                '     py += B * sin(alpha);\n'
//                '}\n')
//
//    return fragment
//
//#inline void cavity(flags, field, frequency, lag):
//#    #field is normalized by pc (reference beam energy)
//#    #frequency is in Hz
//#    omega = '({frequency}) / {speed_of_light}'
//#    return f'{dp += field * sin({2. * np.pi} * ({lag} - ({omega}) * z))};\n'
//
