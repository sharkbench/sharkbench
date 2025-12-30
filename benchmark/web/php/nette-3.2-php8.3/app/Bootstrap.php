<?php

declare(strict_types=1);

namespace App;

use Nette\Bootstrap\Configurator;

class Bootstrap
{
    public static function boot(): Configurator
    {
        $configurator = new Configurator;

        // Disable Tracy in production
        $configurator->setDebugMode(false);

        // Temp directory
        $configurator->setTempDirectory(__DIR__ . '/../temp');

        // Enable RobotLoader
        $configurator->createRobotLoader()
            ->addDirectory(__DIR__)
            ->register();

        // Load configuration
        $configurator->addConfig(__DIR__ . '/../config/common.neon');

        return $configurator;
    }
}
