using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.ValueObjects;
using BerryAIGen.Infrastructure.Data.Repositories;
using FluentAssertions;
using Xunit;

namespace BerryAIGen.IntegrationTests.Infrastructure.Repositories;

/// <summary>
/// Integration tests for ImageRepository.
/// </summary>
public class ImageRepositoryTests : RepositoryTestBase
{
    private readonly ImageRepository _repository;

    public ImageRepositoryTests()
    {
        _repository = new ImageRepository(DbContext);
    }

    [Fact]
    public async Task AddAsync_ShouldAddImageToDatabase()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image = CreateTestImage(folder.Id);

        // Act
        await _repository.AddAsync(image);
        await DbContext.SaveChangesAsync();

        // Assert
        var savedImage = await _repository.GetByIdAsync(image.Id);
        savedImage.Should().NotBeNull();
        savedImage!.Path.Should().Be(image.Path);
        savedImage.Hash.Should().Be(image.Hash);
    }

    [Fact]
    public async Task GetByHashAsync_ShouldReturnCorrectImage()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image = CreateTestImage(folder.Id);
        await _repository.AddAsync(image);
        await DbContext.SaveChangesAsync();

        // Act
        var result = await _repository.GetByHashAsync(image.Hash);

        // Assert
        result.Should().NotBeNull();
        result!.Id.Should().Be(image.Id);
    }

    [Fact]
    public async Task GetByPathAsync_ShouldReturnCorrectImage()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image = CreateTestImage(folder.Id);
        await _repository.AddAsync(image);
        await DbContext.SaveChangesAsync();

        // Act
        var result = await _repository.GetByPathAsync(image.Path);

        // Assert
        result.Should().NotBeNull();
        result!.Id.Should().Be(image.Id);
    }

    [Fact]
    public async Task GetByFolderIdAsync_ShouldReturnAllImagesInFolder()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image1 = CreateTestImage(folder.Id, "image1.png");
        var image2 = CreateTestImage(folder.Id, "image2.png");
        await _repository.AddAsync(image1);
        await _repository.AddAsync(image2);
        await DbContext.SaveChangesAsync();

        // Act
        var results = await _repository.GetByFolderIdAsync(folder.Id);

        // Assert
        results.Should().HaveCount(2);
        results.Should().Contain(i => i.Id == image1.Id);
        results.Should().Contain(i => i.Id == image2.Id);
    }

    [Fact]
    public async Task GetFavoritesAsync_ShouldReturnOnlyFavoriteImages()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var favoriteImage = CreateTestImage(folder.Id, "favorite.png");
        favoriteImage.MarkAsFavorite();
        var normalImage = CreateTestImage(folder.Id, "normal.png");

        await _repository.AddAsync(favoriteImage);
        await _repository.AddAsync(normalImage);
        await DbContext.SaveChangesAsync();

        // Act
        var results = await _repository.GetFavoritesAsync();

        // Assert
        results.Should().HaveCount(1);
        results[0].Id.Should().Be(favoriteImage.Id);
        results[0].IsFavorite.Should().BeTrue();
    }

    [Fact]
    public async Task Update_ShouldModifyExistingImage()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image = CreateTestImage(folder.Id);
        await _repository.AddAsync(image);
        await DbContext.SaveChangesAsync();

        // Act
        image.MarkAsFavorite();
        _repository.Update(image);
        await DbContext.SaveChangesAsync();

        // Assert
        var updatedImage = await _repository.GetByIdAsync(image.Id);
        updatedImage.Should().NotBeNull();
        updatedImage!.IsFavorite.Should().BeTrue();
    }

    [Fact]
    public async Task Delete_ShouldRemoveImageFromDatabase()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image = CreateTestImage(folder.Id);
        await _repository.AddAsync(image);
        await DbContext.SaveChangesAsync();

        // Act
        _repository.Delete(image);
        await DbContext.SaveChangesAsync();

        // Assert
        var deletedImage = await _repository.GetByIdAsync(image.Id);
        deletedImage.Should().BeNull();
    }

    [Fact]
    public async Task GetCountAsync_ShouldReturnCorrectCount()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        await _repository.AddAsync(CreateTestImage(folder.Id, "img1.png"));
        await _repository.AddAsync(CreateTestImage(folder.Id, "img2.png"));
        await _repository.AddAsync(CreateTestImage(folder.Id, "img3.png"));
        await DbContext.SaveChangesAsync();

        // Act
        var count = await _repository.GetCountAsync();

        // Assert
        count.Should().Be(3);
    }

    [Fact]
    public async Task ExistsByHashAsync_ShouldReturnTrueWhenExists()
    {
        // Arrange
        var folder = CreateTestFolder();
        await DbContext.Folders.AddAsync(folder);
        await DbContext.SaveChangesAsync();

        var image = CreateTestImage(folder.Id);
        await _repository.AddAsync(image);
        await DbContext.SaveChangesAsync();

        // Act
        var exists = await _repository.ExistsByHashAsync(image.Hash);

        // Assert
        exists.Should().BeTrue();
    }

    [Fact]
    public async Task ExistsByPathAsync_ShouldReturnFalseWhenNotExists()
    {
        // Arrange
        var nonExistentPath = FilePath.From(@"C:\NonExistent\path.png");

        // Act
        var exists = await _repository.ExistsByPathAsync(nonExistentPath);

        // Assert
        exists.Should().BeFalse();
    }

    // Helper methods
    private static Folder CreateTestFolder()
    {
        return Folder.Create(
            FilePath.From(@"C:\Test\Images"),
            recursive: true);
    }

    private static Image CreateTestImage(FolderId folderId, string fileName = "test.png")
    {
        return Image.Create(
            FilePath.From($@"C:\Test\Images\{fileName}"),
            Hash.From("ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"),
            ImageDimensions.From(1920, 1080),
            fileSize: 1024000,
            folderId,
            "PNG");
    }
}