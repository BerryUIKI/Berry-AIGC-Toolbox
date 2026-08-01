using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace BerryAIGen.App;

/// <summary>
/// Main window of the BerryAIGen application.
/// </summary>
public partial class MainWindow : Window
{
    /// <summary>
    /// Initializes a new instance of the <see cref="MainWindow"/> class.
    /// </summary>
    public MainWindow()
    {
        InitializeComponent();
    }

    /// <summary>
    /// Initializes the component by loading XAML.
    /// </summary>
    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}