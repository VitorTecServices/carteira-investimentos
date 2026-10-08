# 🦀 Carteira de Investimentos Fullstack - Desafio DIO

> **Desenvolvido por VitorTechServices**

Uma aplicação fullstack completa desenvolvida em Rust, combinando back-end (Axum + SQLx) e front-end (Askama + TailwindCSS + HTML), seguindo os requisitos do desafio da Digital Innovation One (DIO).

## 🚀 Funcionalidades Implementadas (Melhorias)

Este projeto foi construído com base no repositório de referência, e adicionei **6 melhorias principais** para demonstrar domínio prático do ecossistema Rukst:

1. **📊 Dashboard com Portfólio**: Visualização do patrimônio total, calculado em tempo real com base nos ativos vinculados à sua conta.
2. **🎒 Sistema Multi-usuário de Carteiras**: Cada usuário tem sua própria carteira. A tabela `portfolio_items` foi adicionada para isolar os ativos de cada conta.
3. **📝 CRUD de Ativos na Interface**: Adicionados formulários em HTML (via Askama) para cadastrar novos tipos de investimentos e adicioná-los ou removê-los do portfólio.
4. **✅ Validações de Negócio**: Validações robustas (ex: senhas compatíveis, tamanho mínimo de texto, valores positivos). Erros mapeados perfeitamente pelo `thiserror`.
5. **✨ Design UI/UX Moderno**: Templates Askama refatorados para utilizar TailwindCSS (Glassmorphism, Dark Mode, animações).
6. **🧹 Código Seguro e Eficiente**: Tratamento correto de Cookies assinados (`SignedCookieJar`), injeção de dependências (`AppState` e `Repository`) nos handlers do Axum.
7. **🧪 Testes Automatizados**: 13 testes (unitários para JWT + integração para API) garantindo o funcionamento correto de autenticação, rotas protegidas e validações.

## 🛠️ Tecnologias Utilizadas

- **Linguagem**: Rust (edição 2024)
- **Framework Web**: [Axum](https://github.com/tokio-rs/axum)
- **Banco de Dados**: PostgreSQL com [SQLx](https://github.com/launchbadge/sqlx) (macros de verificação em tempo de compilação)
- **Templates**: [Askama](https://github.com/djc/askama) (type-safe templates)
- **Autenticação**: `jwt-simple` (Tokens) e `axum-extra` (Signed Cookies)
- **Hashing**: `password-auth` (Argon2id)
- **Frontend**: HTML5, Vanilla JS, TailwindCSS (CDN)

## 📦 Como Rodar o Projeto

### Pré-requisitos
- [Rust](https://rustup.rs/) (versão 1.85 ou superior)
- [Docker](https://docs.docker.com/get-docker/) e Docker Compose

### Passos

1. **Inicie o Banco de Dados (PostgreSQL)**:
   ```bash
   docker compose up -d
   ```
   > O container será criado com as credenciais do `.env` (`depp_admin` / `secure_postgres_pass` / banco `carteira`).

2. **Execute as Migrations**:
   O projeto utiliza `sqlx::migrate!`, então ao rodar o projeto, as tabelas serão criadas automaticamente no PostgreSQL. 
   *(Nota: para compilar com SQLx offline, certifique-se de que o banco está rodando, ou use o modo prepare do sqlx)*.

3. **Inicie o Servidor Web**:
   ```bash
   cargo run
   ```

4. **Acesse a Aplicação**:
   Abra seu navegador em: [http://localhost:3030](http://localhost:3030)

## 🧪 Como Rodar os Testes

O projeto inclui **13 testes** (unitários + integração):

```bash
# Certifique-se de que o PostgreSQL está rodando
docker compose up -d

# Execute todos os testes
cargo test -- --test-threads=1
```

### Cobertura dos testes:
- **JWT**: Geração, verificação e expiração de tokens
- **Registro**: Sucesso, validação de username curto, senhas incompatíveis
- **Login**: Credenciais inválidas retornam 401
- **Autorização**: Rotas protegidas (dashboard, criar ativo) exigem cookie
- **Frontend**: Páginas públicas (login, register) retornam HTML
- **Redirecionamento**: Root (`/`) redireciona para `/login`

## 📂 Estrutura de Diretórios

```
├── Cargo.toml            # Dependências
├── .env                  # Segredos (JWT, Cookie e Conexão BD)
├── compose.yml           # Infra do Postgres
├── migrations/           # Scripts SQL (UP/DOWN)
├── templates/            # Telas Askama (HTML)
├── tests/                # Testes de integração e unitários
│   └── integration_tests.rs
└── src/
    ├── main.rs           # Inicialização do Tokio
    ├── lib.rs            # Módulo biblioteca (expõe crate para testes)
    ├── app.rs            # Configuração do Estado (Axum + SQLx)
    ├── error.rs          # Enum de Erros Customizados (thiserror)
    ├── models.rs         # Structs e Desserializações (Serde)
    ├── repository.rs     # Queries SQL para Assets, Users, Portfolio
    ├── auth/             # Módulo de Autenticação (JWT, Extratores)
    └── routes/           # Rotas da API REST e do Frontend HTML
```

## 🧑‍💻 Desafios Superados

- **Extratores Personalizados (Axum)**: Desenvolvi o extrator `AuthenticatedUser` utilizando `FromRequestParts` para verificar automaticamente a validade do JWT via Cookie assinado em rotas protegidas (Dashboard, Portfolio).
- **Relacionamentos e Agrupamentos em SQL**: A tabela `portfolio_items` exige JOINs complexos calculando o valor total de patrimônio no próprio banco de dados, utilizando `COALESCE` e retornos limpos via `query_as!`.
- **SSR Seguro com Askama**: Envio de dados processados no Backend direto pro Frontend de maneira Type-Safe, impossibilitando erros comuns de propriedades indefinidas.
