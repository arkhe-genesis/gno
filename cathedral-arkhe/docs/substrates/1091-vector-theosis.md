# VectorTheosis 1091.2

## Fórmulas

### TEE (Temporal Embedding Error)

$$
TEE = \frac{\|h_t - \hat{h}_t\|}{\|h_t\| + \epsilon}
$$

O preditor $\hat{h}_t$ usa **Kernel Ridge Regression** (RKHS) com kernel Gaussiano:

$$\hat{h}_t = \sum_{i=1}^{n-1} \alpha_i \cdot K(h_i, \cdot)$$

onde $K(x, y) = \exp\left(-\frac{\|x - y\|^2}{2\sigma^2}\right)$

### Theosis

$$
\Theta = \exp\left(-TEE_{agg} \cdot \varphi^2 \cdot (1 + H_{spectral})\right)
$$

onde $\varphi = \frac{1+\sqrt{5}}{2}$ (proporção áurea), $\varphi^2 \approx 2.618$.

$H_{spectral}$ é a entropia espectral normalizada da covariância:

$$H_s = \frac{-\sum_i p_i \ln p_i}{\ln N}$$

onde $p_i$ são os autovalores positivos da covariância.

### Fadiga (EMA duplo)

$$F_t = 0.7 \cdot E_s(t) + 0.3 \cdot E_l(t) + 0.1 \cdot TEE_t$$

onde $E_s$ e $E_l$ são médias exponenciais com $\alpha_s=0.3$, $\alpha_l=0.05$.

## Janelas Fibonacci

Janela padrão: `(2, 3, 5, 8, 13)` — sequência de Fibonacci.

TEE agregado usa pesos inversamente proporcionais:

$$TEE_{agg} = \frac{\sum_w \frac{TEE_w}{w}}{\sum_w \frac{1}{w}}$$

## Bifurcação

Detectada quando $\text{Var}(TEE_w) > 0.1$ entre janelas — indica
trajetória instável com preditores concorrentes.

## Gate Axiarquia

| Gate | Condição |
|------|-----------|
| OPEN | TEE < 0.01 ∧ Θ > 0.98 |
| CAUTION | TEE > 0.01 ou Θ < 0.98 |
| RESTRICTED | TEE > 0.05 ou Θ < 0.90 |
| LOCKED | TEE > 0.15 ∧ Θ < 0.50 |
| EMERGENCY | TEE > 0.50 ou Θ < 0.01 |
