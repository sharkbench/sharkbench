<?php

use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Request;
use Illuminate\Support\Facades\Route;

Route::get('/api/v1/periodic-table/element', function () {
    $symbol = $_GET['symbol'];
    $response = Http::get('http://127.0.0.1:5002/element.json');
    $data = $response->json($symbol);
    return response()->json($data);
});


Route::get('/api/v1/periodic-table/shells', function () {
    $symbol = $_GET['symbol'];
    $response = Http::get('http://127.0.0.1:5002/shells.json');
    $data = $response->json($symbol);
    return response()->json(["shells" => $data]);
});
