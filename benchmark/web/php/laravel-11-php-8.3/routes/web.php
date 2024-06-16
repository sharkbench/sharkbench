<?php

use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Request;
use Illuminate\Support\Facades\Route;

Route::get('/api/v1/periodic-table/element', function () {
    $symbol = $_GET['symbol'];
    $response = Http::get('http://web-data-source/element.json');
    $data = $response->json($symbol);
    return response()->json($data);
});


Route::get('/api/v1/periodic-table/shells', function () {
    $symbol = $_GET['symbol'];
    $response = Http::get('http://web-data-source/shells.json');
    $data = $response->json($symbol);
    return response()->json(["shells" => $data]);
});
