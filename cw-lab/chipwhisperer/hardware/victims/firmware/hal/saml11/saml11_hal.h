#ifndef SAML11_HAL_H
#define SAML11_HAL_H

void init_uart(void);
void putch(char c);
char getch(void);

void trigger_setup(void);
void trigger_low(void);
void trigger_high(void);

void HW_AES128_Init(void);
void HW_AES128_LoadKey(uint8_t* key);
void HW_AES128_Enc(uint8_t* pt);
void HW_AES128_Enc_pretrigger(uint8_t* pt);
void HW_AES128_Enc_posttrigger(uint8_t* pt);
void HW_AES128_Dec(uint8_t *ct);
#endif
