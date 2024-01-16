FROM julia:1.10

WORKDIR /app

# Dependencies
COPY deps.jl ./
RUN julia deps.jl

COPY app.jl ./

CMD ["julia", "app.jl"]
