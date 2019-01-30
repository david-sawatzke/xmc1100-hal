    .syntax unified
    .cpu cortex-m0
/* Inspired by XMC-for-Arduino */
/* # Default clock values */
    .section ".XmcClockConfig","a",%progbits
    .long  0
    .long HardFault
    /* Clock config */
    /* Leave clock config alone, results in 8 MHz */
    .long 0x00000000
    /* Leave clock gating alone */
    .long 0x80000000
/* ==================VENEERS VENEERS VENEERS VENEERS VENEERS=============== */
    .section ".XmcVeneerCode","ax",%progbits

    .align 1
    .long 0
    .long 0
    .long 0

    .globl HardFault_Veneer
HardFault_Veneer:
    LDR R0, =HardFault
    MOV PC,R0
    .long 0
    .long 0
    .long 0
    .long 0
    .long 0
    .long 0
    .long 0

/* ======================================================================== */
    .globl SVC_Veneer
SVC_Veneer:
    LDR R0, =SVCall
    MOV PC,R0
    .long 0
    .long 0
/* ======================================================================== */
    .globl PendSV_Veneer
PendSV_Veneer:
    LDR R0, =PendSV
    MOV PC,R0
/* ======================================================================== */
    .globl SysTick_Veneer
SysTick_Veneer:
    LDR R0, =SysTick
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_0_Veneer
SCU_0_Veneer:
    LDR R0, =SCU_0
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_1_Veneer
SCU_1_Veneer:
    LDR R0, =SCU_1
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_2_Veneer
SCU_2_Veneer:
    LDR R0, =SCU_2
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_3_Veneer
SCU_3_Veneer:
    LDR R0, =ERU0_0
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_4_Veneer
SCU_4_Veneer:
    LDR R0, =ERU0_1
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_5_Veneer
SCU_5_Veneer:
    LDR R0, =ERU0_2
    MOV PC,R0
/* ======================================================================== */
    .globl SCU_6_Veneer
SCU_6_Veneer:
    LDR R0, =ERU0_3
    MOV PC,R0
    .long 0
    .long 0
/* ======================================================================== */
    .globl USIC0_0_Veneer
USIC0_0_Veneer:
    LDR R0, =USIC0_0
    MOV PC,R0
/* ======================================================================== */
    .globl USIC0_1_Veneer
USIC0_1_Veneer:
    LDR R0, =USIC0_1
    MOV PC,R0
/* ======================================================================== */
    .globl USIC0_2_Veneer
USIC0_2_Veneer:
    LDR R0, =USIC0_2
    MOV PC,R0
/* ======================================================================== */
    .globl USIC0_3_Veneer
USIC0_3_Veneer:
    LDR R0, =USIC0_3
    MOV PC,R0
/* ======================================================================== */
    .globl USIC0_4_Veneer
USIC0_4_Veneer:
    LDR R0, =USIC0_4
    MOV PC,R0
/* ======================================================================== */
    .globl USIC0_5_Veneer
USIC0_5_Veneer:
    LDR R0, =USIC0_5
    MOV PC,R0
/* ======================================================================== */
    .globl VADC0_C0_0_Veneer
VADC0_C0_0_Veneer:
    LDR R0, =VADC0_C0_0
    MOV PC,R0
/* ======================================================================== */
    .globl VADC0_C0_1_Veneer
VADC0_C0_1_Veneer:
    LDR R0, =VADC0_C0_1
    MOV PC,R0
    .long 0
    .long 0
    .long 0
    .long 0
/* ======================================================================== */
    .globl CCU40_0_Veneer
CCU40_0_Veneer:
    LDR R0, =CCU40_0
    MOV PC,R0
/* ======================================================================== */
    .globl CCU40_1_Veneer
CCU40_1_Veneer:
    LDR R0, =CCU40_1
    MOV PC,R0
/* ======================================================================== */
    .globl CCU40_2_Veneer
CCU40_2_Veneer:
    LDR R0, =CCU40_2
    MOV PC,R0
/* ======================================================================== */
    .globl CCU40_3_Veneer
CCU40_3_Veneer:
    LDR R0, =CCU40_3
    MOV PC,R0
    .long 0
    .long 0
    .long 0
    .long 0
    .long 0
    .long 0
    .long 0

/* ======================================================================== */
/* ======================================================================== */

/* ============= END OF INTERRUPT HANDLER DEFINITION ======================== */
    .end
