using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;

namespace BerryAIGen.App;

/// <summary>
/// Main application class for BerryAIGen.
/// </summary>
public class App : Application
{
    /// <summary>
    /// Initializes the application by loading XAML resources.
    /// </summary>
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    /// <summary>
    /// Called when the application framework initialization is completed.
    /// </summary>
    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.MainWindow = new MainWindow();
        }

        base.OnFrameworkInitializationCompleted();
    }
}